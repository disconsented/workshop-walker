use std::sync::OnceLock;

use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort, async_trait, call};
use salvo::{
    Depot, Writer,
    oapi::{endpoint, extract::PathParam},
    prelude::{Json, StatusCode, StatusError},
};
use surrealdb::{Surreal, engine::local::Db};
use surrealdb_core::sql::{
    BinaryOperator, Closure, Cond, Dir, Expr, Field, Fields, Idiom, Kind, Literal, Lookup, Param,
    Part, RecordIdKeyLit, RecordIdLit,
    field::Selector,
    literal::ObjectEntry,
    lookup::{LookupKind, LookupSubject},
    part::DestructurePart,
    statements::SelectStatement,
};
use surrealdb_types::{RecordId, SurrealValue, ToSql};
use tracing::{debug, error, instrument};

use crate::{
    db::{
        IItemID, IUserID,
        model::{ExternalFullWorkshopItem, InternalFullWorkshopItem, Status},
    },
    web::auth,
};

static ITEM_ACTOR: OnceLock<ActorRef<ItemMsg>> = OnceLock::new();

pub type Result<T, E = Error> = std::result::Result<T, E>;
pub type Error = StatusError;

#[derive(Debug)]
enum InnerError {
    NotFound,
    InternalError,
}

impl InnerError {
    fn status_code(&self) -> StatusCode {
        match self {
            InnerError::NotFound => StatusCode::NOT_FOUND,
            InnerError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<InnerError> for StatusError {
    fn from(value: InnerError) -> Self {
        let mut error = StatusError::internal_server_error();
        error.code = value.status_code();
        error.name = value
            .status_code()
            .canonical_reason()
            .unwrap_or_default()
            .to_string();
        error.brief = format!("{value:?}");
        error.detail = None;
        error
    }
}

pub struct ItemActor;
pub struct ItemState {
    database: Surreal<Db>,
}
pub struct ItemArgs {
    pub database: Surreal<Db>,
}

pub enum ItemMsg {
    Get(
        IItemID,
        Option<IUserID>,
        RpcReplyPort<Result<InternalFullWorkshopItem>>,
    ),
}

#[async_trait]
impl Actor for ItemActor {
    type Arguments = ItemArgs;
    type Msg = ItemMsg;
    type State = ItemState;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        ITEM_ACTOR.get_or_init(|| myself);
        Ok(ItemState {
            database: args.database,
        })
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            ItemMsg::Get(id, user, reply) => {
                let res = get_item(&state.database, id, user).await;
                if reply.send(res).is_err() {
                    error!(message = "Get", "Failed to reply to message");
                }
            }
        }
        Ok(())
    }
}

async fn get_item(
    db: &Surreal<Db>,
    id: IItemID,
    user: Option<IUserID>,
) -> Result<InternalFullWorkshopItem> {
    let mut prop_fields = vec![
        DestructurePart::Field("in".into()),
        DestructurePart::Field("id".into()),
        DestructurePart::Field("source".into()),
        DestructurePart::Field("status".into()),
        DestructurePart::Field("upvote_count".into()),
        DestructurePart::Field("vote_count".into()),
        DestructurePart::Aliased(
            "out".into(),
            Idiom(vec![
                Part::Field("out".into()),
                Part::Method("id".into(), vec![]),
            ]),
        ),
    ];

    // The current user's own vote score for this property, if any. Looks up the
    // per-user vote record keyed the same way the write side keys it (see
    // `properties_repository::vote`): votes:{ link, user, item }. `.score` on a
    // record id that doesn't exist yields NONE, so un-voted properties come
    // back as `None`. Only computable when we know who the caller is.
    if let Some(user) = &user {
        prop_fields.push(DestructurePart::Aliased(
            "vote_state".into(),
            Idiom(vec![
                Part::Start(Expr::Literal(Literal::RecordId(RecordIdLit {
                    table: "votes".into(),
                    key: RecordIdKeyLit::Object(vec![
                        ObjectEntry {
                            key: "link".into(),
                            // The property record this edge points at.
                            value: Expr::Idiom(Idiom(vec![Part::Field("out".into())])),
                        },
                        ObjectEntry {
                            key: "user".into(),
                            value: Expr::from_public_value(user.clone().into_value()),
                        },
                        ObjectEntry {
                            key: "item".into(),
                            // The workshop item this edge originates from.
                            value: Expr::Idiom(Idiom(vec![Part::Field("in".into())])),
                        },
                    ]),
                }))),
                Part::Field("score".into()),
            ]),
        ));
    }
    let dep_fields = [
        DestructurePart::Field("app".into()),
        DestructurePart::All("author".into()),
        DestructurePart::Field("description".into()),
        DestructurePart::Field("id".into()),
        DestructurePart::Field("languages".into()),
        DestructurePart::Field("last_updated".into()),
        DestructurePart::Field("preview_url".into()),
        DestructurePart::Field("score".into()),
        DestructurePart::Field("title".into()),
        DestructurePart::Aliased(
            "tags".into(),
            Idiom(vec![
                Part::Field("tags".into()),
                Part::Method(
                    "filter".into(),
                    vec![Expr::Closure(Box::new(Closure {
                        args: vec![(Param::new("tag".to_string()), Kind::Any)],
                        returns: None,
                        body: Expr::Idiom(Idiom(vec![
                            Part::Start(Expr::Param(Param::new("tag".to_string()))),
                            Part::Method("exists".into(), vec![]),
                        ])),
                    }))],
                ),
                Part::All,
            ]),
        ),
    ];
    let mut stmt = SelectStatement::default();
    stmt.what = vec![Expr::from_public_value(
        RecordId::from(id.clone()).into_value(),
    )];

    // Keep accepted properties, plus (for a signed-in user) their own submitted
    // ones regardless of status. The condition goes on the graph lookup itself:
    // a `.filter()` after the lookup binds to each edge rather than to the
    // array, and the parentheses which would fix that are lost when the driver
    // prints the statement back to SQL.
    let status_accepted = Expr::Binary {
        left: Box::new(Expr::Idiom(Idiom::field("status".to_string()))),
        op: BinaryOperator::ExactEqual,
        right: Box::new(Expr::Literal(Literal::Integer(Status::Accepted as i64))),
    };
    let properties_condition = if let Some(user) = user {
        Expr::Binary {
            left: Box::new(status_accepted),
            op: BinaryOperator::Or,
            right: Box::new(Expr::Binary {
                left: Box::new(Expr::Idiom(Idiom::field("source".to_string()))),
                op: BinaryOperator::ExactEqual,
                right: Box::new(Expr::from_public_value(user.into_value())),
            }),
        }
    } else {
        status_accepted
    };

    stmt.fields = Fields::Select(vec![
        Field::All,
        Field::Single(Selector {
            expr: Expr::Idiom(Idiom(vec![
                Part::Field("tags".into()),
                Part::Method(
                    "filter".into(),
                    vec![Expr::Closure(Box::new(Closure {
                        args: vec![(Param::new("tag".to_string()), Kind::Any)],
                        returns: None,
                        body: Expr::Idiom(Idiom(vec![
                            Part::Start(Expr::Param(Param::new("tag".to_string()))),
                            Part::Method("exists".into(), vec![]),
                        ])),
                    }))],
                ),
                Part::All,
            ])),
            alias: None,
        }),
        Field::Single(Selector {
            expr: Expr::Idiom(Idiom(vec![Part::Field("author".into()), Part::All])),
            alias: None,
        }),
        Field::Single(Selector {
            // `array::filter`, not a `.filter()` method part: the driver prints
            // the statement back to SQL and re-parses it, and the printer drops
            // the parentheses a method call on the whole lookup needs. Without
            // them the filter binds to each element and the query fails; in an
            // alias (a destination path, not an expression) it is dropped
            // without an error.
            expr: Expr::Idiom(Idiom(vec![
                Part::Graph(Box::new(Lookup {
                    kind: LookupKind::Graph(Dir::Out),
                    what: vec![LookupSubject::Table {
                        table: "workshop_item_properties".into(),
                        referencing_field: None,
                    }],
                    cond: Some(Cond(properties_condition)),
                    ..Default::default()
                })),
                Part::Destructure(prop_fields.clone()),
            ])),
            alias: Some(Idiom::field("properties".to_string())),
        }),
        Field::Single(Selector {
            expr: Expr::Idiom(Idiom(vec![
                Part::Graph(Box::new(Lookup {
                    kind: LookupKind::Graph(Dir::Out),
                    what: vec![LookupSubject::Table {
                        table: "item_dependencies".into(),
                        referencing_field: None,
                    }],
                    ..Default::default()
                })),
                Part::All,
                Part::Field("out".into()),
                Part::All,
                Part::Destructure(dep_fields.to_vec()),
            ])),
            alias: Some(Idiom::field("dependencies".to_string())),
        }),
        Field::Single(Selector {
            expr: Expr::Idiom(Idiom(vec![
                Part::Graph(Box::new(Lookup {
                    kind: LookupKind::Graph(Dir::In),
                    what: vec![LookupSubject::Table {
                        table: "item_dependencies".into(),
                        referencing_field: None,
                    }],
                    ..Default::default()
                })),
                Part::All,
                Part::Field("in".into()),
                Part::All,
                Part::Destructure(dep_fields.to_vec()),
            ])),
            alias: Some(Idiom::field("dependants".to_string())),
        }),
    ]);

    debug!(sql = stmt.to_sql(), "item query");
    let mut thing = db
        .query(stmt)
        .bind(("id", RecordId::from(id)))
        .await
        .inspect_err(|error| error!(message = "get_item", ?error, "Failed to query database"))
        .map_err(|_| InnerError::InternalError)?;

    let result: Option<InternalFullWorkshopItem> = thing
        .take(0)
        .inspect_err(|error| error!(message = "get_item", ?error, "Failed to take result"))
        .map_err(|_| InnerError::InternalError)?;
    result.ok_or(Error::from(InnerError::NotFound))
}

/// GET /api/item/{id}
/// Retrieves a full workshop item by id, including dependencies and dependants.
#[endpoint]
#[instrument(skip_all)]
pub async fn get(id: PathParam<i64>, depot: &mut Depot) -> Result<Json<ExternalFullWorkshopItem>> {
    // Lazily spawn the actor on first use and keep a global reference like
    // auth.rs
    let actor = ITEM_ACTOR.get().cloned().ok_or(InnerError::InternalError)?;

    let user = auth::get_user_from_depot(depot);
    let data = call!(actor, |reply| { ItemMsg::Get(id.0.into(), user, reply) })
        .map_err(|_| InnerError::InternalError)??;
    Ok(Json(
        data.try_into().map_err(|_| InnerError::InternalError)?,
    ))
}

#[cfg(test)]
mod test {
    use surrealdb::{Surreal, engine::local::Mem};

    use super::{Db, InternalFullWorkshopItem, get_item};
    use crate::db::{IItemID, IUserID};

    /// Stand up an in-memory database with just enough schema for `get_item`,
    /// and wire up two dependency edges around item 100:
    ///   - 100 -> item_dependencies -> 200   (100 depends on 200)
    ///   - 300 -> item_dependencies -> 100   (300 depends on 100)
    /// So from item 100's perspective: 200 is a dependency, 300 is a dependant.
    async fn seed_db() -> Surreal<Db> {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();

        db.query(
            "
            DEFINE TABLE apps TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
            DEFINE FIELD id ON apps TYPE int PERMISSIONS FULL;
            DEFINE FIELD name ON apps TYPE string PERMISSIONS FULL;

            DEFINE TABLE usernames TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
            DEFINE FIELD id ON usernames TYPE int PERMISSIONS FULL;
            DEFINE FIELD name ON usernames TYPE string PERMISSIONS FULL;

            DEFINE TABLE tags TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
            DEFINE FIELD id ON tags TYPE string PERMISSIONS FULL;
            DEFINE FIELD display_name ON tags TYPE string PERMISSIONS FULL;

            DEFINE TABLE properties TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
            DEFINE FIELD id ON properties TYPE { class: string, value: string } PERMISSIONS FULL;

            DEFINE TABLE workshop_items TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
            DEFINE FIELD id ON workshop_items TYPE int PERMISSIONS FULL;
            DEFINE FIELD app ON workshop_items TYPE record<apps> PERMISSIONS FULL;
            DEFINE FIELD author ON workshop_items TYPE record<usernames> PERMISSIONS FULL;
            DEFINE FIELD description ON workshop_items TYPE string PERMISSIONS FULL;
            DEFINE FIELD languages ON workshop_items TYPE array<int> PERMISSIONS FULL;
            DEFINE FIELD last_updated ON workshop_items TYPE int PERMISSIONS FULL;
            DEFINE FIELD preview_url ON workshop_items TYPE none | string PERMISSIONS FULL;
            DEFINE FIELD score ON workshop_items TYPE float PERMISSIONS FULL;
            DEFINE FIELD tags ON workshop_items TYPE array<record<tags>> PERMISSIONS FULL;
            DEFINE FIELD title ON workshop_items TYPE string PERMISSIONS FULL;

            DEFINE TABLE item_dependencies TYPE RELATION IN workshop_items OUT workshop_items \
             SCHEMAFULL PERMISSIONS NONE;
            DEFINE TABLE workshop_item_properties TYPE RELATION IN workshop_items OUT properties \
             SCHEMAFULL PERMISSIONS NONE;

            CREATE apps:1 SET id = 1, name = 'Test App';
            CREATE usernames:1 SET id = 1, name = 'Test Author';
            CREATE tags:test SET id = 'test', display_name = 'Test Tag';

            CREATE workshop_items:100 SET id = 100, app = apps:1, author = usernames:1, \
             description = 'item 100', languages = [], last_updated = 0, score = 1.0f, tags = \
             [tags:test], title = 'Item 100';
            CREATE workshop_items:200 SET id = 200, app = apps:1, author = usernames:1, \
             description = 'item 200', languages = [], last_updated = 0, score = 1.0f, tags = \
             [tags:test], title = 'Item 200';
            CREATE workshop_items:300 SET id = 300, app = apps:1, author = usernames:1, \
             description = 'item 300', languages = [], last_updated = 0, score = 1.0f, tags = \
             [tags:test], title = 'Item 300';

            RELATE workshop_items:100 -> item_dependencies -> workshop_items:200;
            RELATE workshop_items:300 -> item_dependencies -> workshop_items:100;

            DEFINE TABLE users TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
            DEFINE FIELD id ON users TYPE int PERMISSIONS FULL;

            DEFINE TABLE votes TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
            DEFINE FIELD id ON votes TYPE { item: record<workshop_items>, link: \
             record<properties>, user: record<users> } PERMISSIONS FULL;
            DEFINE FIELD score ON votes TYPE int PERMISSIONS FULL;
            DEFINE FIELD user ON votes TYPE record<users> PERMISSIONS FULL;
            DEFINE FIELD when ON votes TYPE datetime PERMISSIONS FULL;

            DEFINE FIELD note ON workshop_item_properties TYPE none | string PERMISSIONS FULL;
            DEFINE FIELD source ON workshop_item_properties TYPE 'system' | record<users> \
             PERMISSIONS FULL;
            DEFINE FIELD status ON workshop_item_properties TYPE -1 | 0 | 1 DEFAULT 0 PERMISSIONS \
             FULL;
            DEFINE FIELD upvote_count ON workshop_item_properties TYPE int DEFAULT 0 PERMISSIONS \
             FULL;
            DEFINE FIELD vote_count ON workshop_item_properties TYPE int DEFAULT 0 PERMISSIONS \
             FULL;

            CREATE users:1 SET id = 1;
            CREATE users:2 SET id = 2;
            CREATE properties:{ class: 'Type', value: 'test' };
            CREATE properties:{ class: 'Type', value: 'pending' };
            CREATE properties:{ class: 'Type', value: 'rejected' };

            -- Accepted property on item 100 so it survives the status filter.
            RELATE workshop_items:100 -> workshop_item_properties -> properties:{ class: 'Type', \
             value: 'test' } SET status = 1, source = 'system', upvote_count = 1, vote_count = 1;

            -- Pending submission from user 2, and a rejected one. Only user 2
            -- sees the pending entry; nobody sees the rejected one.
            RELATE workshop_items:100 -> workshop_item_properties -> properties:{ class: 'Type', \
             value: 'pending' } SET status = 0, source = users:2;
            RELATE workshop_items:100 -> workshop_item_properties -> properties:{ class: 'Type', \
             value: 'rejected' } SET status = -1, source = 'system';

            -- User 1 has voted +1 on that property; user 2 has not voted.
            CREATE votes:{ item: workshop_items:100, link: properties:{ class: 'Type', value: \
             'test' }, user: users:1 } SET score = 1, user = users:1, when = time::now();
            ",
        )
        .await
        .unwrap()
        .check()
        .unwrap();

        db
    }

    /// Regression test for the dependency/dependant ordering bug
    /// (commit 4269efc): the `dependencies` selector traverses outgoing edges
    /// and must read the edge's `out` node (the thing depended upon), not `in`
    /// (the item itself). Reading the wrong end made an item report itself as
    /// its own dependency and dropped its real dependencies.
    #[tokio::test]
    async fn dependencies_and_dependants_resolve_to_correct_ends() {
        let db = seed_db().await;

        let item = get_item(&db, IItemID::from(100i64), None)
            .await
            .expect("item 100 should be found");

        // Outgoing edge 100 -> 200 means 200 is a dependency of 100.
        let dependency_ids: Vec<IItemID> =
            item.dependencies.iter().map(|dep| dep.id.clone()).collect();
        assert_eq!(
            dependency_ids,
            vec![IItemID::from(200i64)],
            "dependencies should be the `out` end of outgoing edges (200), not the item itself"
        );

        // Incoming edge 300 -> 100 means 300 is a dependant of 100.
        let dependant_ids: Vec<IItemID> =
            item.dependants.iter().map(|dep| dep.id.clone()).collect();
        assert_eq!(
            dependant_ids,
            vec![IItemID::from(300i64)],
            "dependants should be the `in` end of incoming edges (300)"
        );
    }

    /// A leaf item with only an incoming edge should report a dependant and no
    /// dependencies, guarding against the two directions being swapped.
    #[tokio::test]
    async fn dependency_only_item_reports_no_dependants() {
        let db = seed_db().await;

        let item = get_item(&db, IItemID::from(200i64), None)
            .await
            .expect("item 200 should be found");

        assert!(
            item.dependencies.is_empty(),
            "item 200 has no outgoing edges, so it has no dependencies"
        );
        let dependant_ids: Vec<IItemID> =
            item.dependants.iter().map(|dep| dep.id.clone()).collect();
        assert_eq!(
            dependant_ids,
            vec![IItemID::from(100i64)],
            "item 200 is depended upon by 100"
        );
    }

    /// Regression test: `vote_state` on a property should reflect the current
    /// user's own vote score. User 1 voted +1 on item 100's property, so
    /// fetching item 100 as user 1 must surface `Some(1)` — guarding against
    /// the projection dropping the per-user vote lookup (it silently
    /// returned `None` after the raw-SQL query was ported to the AST
    /// builder).
    #[tokio::test]
    async fn vote_state_reflects_current_users_vote() {
        let db = seed_db().await;

        let item = get_item(&db, IItemID::from(100i64), Some(IUserID::from(1i64)))
            .await
            .expect("item 100 should be found");

        let prop = item
            .properties
            .iter()
            .find(|prop| prop.out.value == "test")
            .expect("item 100 has an accepted property");
        assert_eq!(
            prop.vote_state,
            Some(1),
            "user 1 voted +1 on this property, so vote_state should be Some(1)"
        );
    }

    /// A user who hasn't voted (user 2) — and the anonymous case (no user) —
    /// should see `vote_state == None`, since the vote record id won't resolve.
    #[tokio::test]
    async fn vote_state_none_when_user_has_not_voted() {
        let db = seed_db().await;

        let as_other_user = get_item(&db, IItemID::from(100i64), Some(IUserID::from(2i64)))
            .await
            .expect("item 100 should be found");
        assert_eq!(
            as_other_user
                .properties
                .iter()
                .find(|prop| prop.out.value == "test")
                .expect("item 100 has an accepted property")
                .vote_state,
            None,
            "user 2 has not voted, so vote_state should be None"
        );

        let anonymous = get_item(&db, IItemID::from(100i64), None)
            .await
            .expect("item 100 should be found");
        assert_eq!(
            anonymous
                .properties
                .iter()
                .find(|prop| prop.out.value == "test")
                .expect("item 100 has an accepted property")
                .vote_state,
            None,
            "anonymous request has no user, so vote_state should be None"
        );
    }

    /// Regression test: `get_item` must only return accepted properties for an
    /// anonymous caller. Pending and rejected edges leaked through when the
    /// status filter was attached to the projection's alias instead of its
    /// expression, so the filter was parsed but never evaluated.
    #[tokio::test]
    async fn anonymous_item_only_returns_accepted_properties() {
        let db = seed_db().await;

        let item = get_item(&db, IItemID::from(100i64), None)
            .await
            .expect("item 100 should be found");

        assert_eq!(
            property_values(&item),
            vec!["test".to_string()],
            "anonymous callers must not see pending or rejected properties"
        );
    }

    /// A signed-in user sees accepted properties plus their own submissions,
    /// whatever the status, and still never sees somebody else's pending work.
    #[tokio::test]
    async fn item_returns_accepted_properties_and_own_submissions() {
        let db = seed_db().await;

        let as_submitter = get_item(&db, IItemID::from(100i64), Some(IUserID::from(2i64)))
            .await
            .expect("item 100 should be found");
        assert_eq!(
            property_values(&as_submitter),
            vec!["pending".to_string(), "test".to_string()],
            "user 2 should see the accepted property and their own pending one"
        );

        let as_other_user = get_item(&db, IItemID::from(100i64), Some(IUserID::from(1i64)))
            .await
            .expect("item 100 should be found");
        assert_eq!(
            property_values(&as_other_user),
            vec!["test".to_string()],
            "user 1 submitted nothing, so they only see the accepted property"
        );
    }

    /// The property values on an item, sorted so the assertions do not depend
    /// on edge order.
    fn property_values(item: &InternalFullWorkshopItem) -> Vec<String> {
        let mut values: Vec<String> = item
            .properties
            .iter()
            .map(|prop| prop.out.value.clone())
            .collect();
        values.sort();
        values
    }
}
