use salvo::{
    Depot, Request, Writer,
    oapi::{ToSchema, endpoint},
    prelude::{Json, ToParameters},
};
use serde::{
    Deserialize, Deserializer,
    de::{Error, IntoDeserializer},
};
use snafu::{ResultExt, Whatever};
use surrealdb::{Surreal, engine::local::Db};
use surrealdb_core::sql::{
    BinaryOperator, Closure, Cond, Dir, Expr, Field, Fields, Idiom, Kind, Limit, Literal, Lookup,
    Order, Param, Part, RecordIdKeyLit, RecordIdLit, Start,
    field::Selector,
    literal::ObjectEntry,
    lookup::{LookupKind, LookupSubject},
    operator::MatchesOperator,
    order::{OrderList, Ordering},
    part::DestructurePart,
    statements::SelectStatement,
};
use surrealdb_types::{RecordId, SurrealValue, ToSql};
use tracing::{Instrument, debug, info_span, instrument, trace};

use crate::{
    db::{
        IAppID, IPropertyID, ITagID, IUserID,
        model::{Class, ExternalWorkshopItem, InternalWorkshopItem, OrderBy, Property, Status},
    },
    processing::language_actor::DetectedLanguage,
    web,
    web::{DB_POOL, auth},
};

#[derive(ToSchema, Debug)]
struct PropertyParam(pub Property);
impl<'de> Deserialize<'de> for PropertyParam {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        let (class, value) = raw
            .split_once(':')
            .ok_or(D::Error::custom("expected `:` found none"))?;
        let class = Class::deserialize(class.into_deserializer())?;
        Ok(Self(Property {
            class,
            value: value.to_string(),
        }))
    }
}

#[derive(Deserialize, ToParameters, Debug)]
struct Parameters {
    app: i64,
    page: Option<u64>,
    limit: Option<u64>,
    language: Option<DetectedLanguage>,
    tags: Option<Vec<String>>,
    title: Option<String>,
    updated_before: Option<i64>,
    updated_after: Option<i64>,
    order_by: Option<OrderBy>,
    positive_props: Option<Vec<PropertyParam>>,
    negative_props: Option<Vec<PropertyParam>>,
}

// ToDo: Seperate out filtering to its own struct
// And, handle pagination based on the last element for performance
#[instrument(skip_all)]
#[endpoint]
pub async fn list(
    _: &mut Request,
    depot: &mut Depot,
    parameters: Parameters,
) -> web::Result<Json<Vec<ExternalWorkshopItem>>> {
    let Parameters {
        app,
        page,
        limit,
        language,
        mut tags,
        mut title,
        updated_before,
        updated_after,
        mut order_by,
        positive_props,
        negative_props,
    } = parameters;
    let page = page.unwrap_or(0);
    let limit = limit.unwrap_or(100).min(100);
    let db: &Surreal<Db> = DB_POOL.get().expect("Getting db connection");
    let user = auth::get_user_from_depot(depot);
    let positive_props = {
        let mut props = positive_props.unwrap_or_default();
        props
            .drain(..props.len().min(5))
            .map(|param| param.0)
            .collect()
    };
    let negative_props = {
        let mut props = negative_props.unwrap_or_default();
        props
            .drain(..props.len().min(5))
            .map(|param| param.0)
            .collect()
    };

    let results = query_inner(
        app,
        page,
        limit,
        language,
        tags.take().unwrap_or_default(),
        title.take(),
        updated_before,
        updated_after,
        order_by.take(),
        positive_props,
        negative_props,
        db,
        user,
    )
    .instrument(info_span!("query list").or_current())
    .await?;

    Ok(Json(results))
}

#[instrument(skip_all)]
async fn query_inner(
    app: i64,
    page: u64,
    limit: u64,
    language: Option<DetectedLanguage>,
    tags: Vec<String>,
    title: Option<String>,
    updated_before: Option<i64>,
    updated_after: Option<i64>,
    order_by: Option<OrderBy>,
    positive_props: Vec<Property>,
    negative_props: Vec<Property>,
    db: &Surreal<Db>,
    user: Option<IUserID>,
) -> web::Result<Vec<ExternalWorkshopItem>, Whatever> {
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

    // The current user's own vote score for each property, if any. Mirrors
    // `item::get_item`: looks up the per-user vote record keyed the same way
    // the write side keys it (see `properties_repository::vote`): votes:{
    // link, user, item }. `.score` on a record id that doesn't exist yields
    // NONE, so un-voted properties come back as `None`. Only computable
    // with a user.
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

    let status_accepted = Expr::Binary {
        left: Box::new(Expr::Idiom(Idiom::field("status".to_string()))),
        op: BinaryOperator::ExactEqual,
        right: Box::new(Expr::Literal(Literal::Integer(Status::Accepted as i64))),
    };
    let properties_condition = if let Some(user) = &user {
        Expr::Binary {
            left: Box::new(status_accepted),
            op: BinaryOperator::Or,
            right: Box::new(Expr::Binary {
                left: Box::new(Expr::Idiom(Idiom::field("source".to_string()))),
                op: BinaryOperator::ExactEqual,
                right: Box::new(Expr::from_public_value(user.clone().into_value())),
            }),
        }
    } else {
        status_accepted
    };

    let app = IAppID::from(app);
    let mut stmt = SelectStatement {
        what: vec![Expr::Table("workshop_items".into())],
        ..Default::default()
    };
    {
        stmt.fields = Fields::Select(vec![
            Field::All,
            Field::Single(Selector {
                expr: Expr::Idiom(Idiom(vec![
                    Part::Graph(Box::from(Lookup {
                        kind: LookupKind::Graph(Dir::Out),
                        what: vec![LookupSubject::Table {
                            table: "workshop_item_properties".into(),
                            referencing_field: None,
                        }],
                        cond: Some(Cond(properties_condition.clone())),
                        ..Default::default()
                    })),
                    Part::Destructure(prop_fields.clone()),
                ])),
                alias: Some(Idiom::field("properties".to_string())),
            }),
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
            // Author's are more so considered eventually consistent
            Field::Single(Selector {
                expr: Expr::Idiom(Idiom(vec![Part::Field("author".into()), Part::All])),
                alias: None,
            }),
        ]);
    }
    stmt.limit = Some(Limit(Expr::from_public_value(limit.into_value())));
    stmt.start = Some(Start(Expr::from_public_value((page * limit).into_value())));
    stmt.cond = {
        let mut conditions = vec![];
        conditions.push(Expr::Binary {
            left: Box::new(Expr::Idiom(Idiom::field("app".to_string()))),
            op: BinaryOperator::Equal,
            right: Box::new(Expr::from_public_value(RecordId::from(app).into_value())),
        });

        if let Some(language) = language {
            // If we got back to supporting multiple languages this needs to go
            // back to ContainAny Otherwise, it kinda breaks
            conditions.push(Expr::Binary {
                left: Box::new(Expr::Idiom(Idiom::field("languages".to_string()))),
                op: BinaryOperator::Contain,
                right: Box::new(Expr::Literal(Literal::Integer(language as i64))),
            });
        }

        if !tags.is_empty() {
            conditions.push(Expr::Binary {
                left: Box::new(Expr::Idiom(Idiom::field("tags".to_string()))),
                op: BinaryOperator::ContainAll,
                right: Box::new(Expr::Literal(Literal::Array(
                    tags.into_iter()
                        .map(|tag| Expr::from_public_value(ITagID::from(tag).into_value()))
                        .collect::<Vec<_>>(),
                ))),
            });
        }

        if !positive_props.is_empty() {
            let idiom = Expr::Idiom(Idiom(vec![
                Part::Graph(Box::from(Lookup {
                    kind: LookupKind::Graph(Dir::Out),
                    what: vec![LookupSubject::Table {
                        table: "workshop_item_properties".into(),
                        referencing_field: None,
                    }],
                    cond: Some(Cond(properties_condition.clone())),
                    ..Default::default()
                })),
                Part::Field("out".into()),
            ]));
            conditions.push(Expr::Binary {
                left: Box::new(idiom),
                op: BinaryOperator::ContainAll,
                right: Box::new(Expr::Literal(Literal::Array(
                    positive_props
                        .into_iter()
                        .map(|prop| Expr::from_public_value(IPropertyID::from(prop).into_value()))
                        .collect::<Vec<_>>(),
                ))),
            });
        }

        if !negative_props.is_empty() {
            let idiom = Expr::Idiom(Idiom(vec![
                Part::Graph(Box::from(Lookup {
                    kind: LookupKind::Graph(Dir::Out),
                    what: vec![LookupSubject::Table {
                        table: "workshop_item_properties".into(),
                        referencing_field: None,
                    }],
                    cond: Some(Cond(properties_condition.clone())),
                    ..Default::default()
                })),
                Part::Field("out".into()),
            ]));
            conditions.push(Expr::Binary {
                left: Box::new(idiom),
                op: BinaryOperator::ContainNone,
                right: Box::new(Expr::Literal(Literal::Array(
                    negative_props
                        .into_iter()
                        .map(|prop| Expr::from_public_value(IPropertyID::from(prop).into_value()))
                        .collect::<Vec<_>>(),
                ))),
            });
        }

        if let Some(title) = title {
            conditions.push(Expr::Binary {
                left: Box::new(Expr::Idiom(Idiom::field("title".to_string()))),
                op: BinaryOperator::Matches(MatchesOperator {
                    rf: None,
                    operator: None,
                }),
                right: Box::new(Expr::Literal(Literal::String(title.into()))),
            });
        }

        if let Some(last_updated) = updated_before {
            conditions.push(Expr::Binary {
                left: Box::new(Expr::Idiom(Idiom::field("last_updated".to_string()))),
                op: BinaryOperator::LessThan,
                right: Box::new(Expr::Literal(Literal::Integer(last_updated))),
            });
        }

        if let Some(last_updated) = updated_after {
            conditions.push(Expr::Binary {
                left: Box::new(Expr::Idiom(Idiom::field("last_updated".to_string()))),
                op: BinaryOperator::MoreThan,
                right: Box::new(Expr::Literal(Literal::Integer(last_updated))),
            });
        }

        let first = conditions
            .pop()
            .expect("Expected at least one condition to be present");
        Some(Cond(conditions.into_iter().fold(first, |old, next| {
            Expr::Binary {
                left: Box::new(old),
                op: BinaryOperator::And,
                right: Box::new(next),
            }
        })))
    };

    stmt.order = order_by.map(|order_by| {
        Ordering::Order(OrderList(vec![Order {
            value: Idiom::field(order_by.column_name().to_string()),
            collate: false,
            numeric: false,
            direction: false,
        }]))
    });

    debug!(sql = stmt.to_sql(), "running big query");
    let mut results = db.query(stmt).await.whatever_context("querying")?;

    trace!(?results, "results");

    let results: Vec<InternalWorkshopItem> = results.take(0).whatever_context("taking result")?;

    results
        .into_iter()
        .map(ExternalWorkshopItem::try_from)
        .collect::<Result<_, _>>()
        .whatever_context("converting internal to external")
}

#[cfg(test)]
mod test {
    use surrealdb::{Surreal, engine::local::Mem};

    use super::{Db, query_inner};
    use crate::db::{
        IUserID,
        model::{Class, ExternalSource, Status},
    };

    /// In-memory database with one app, one item, and three property edges on
    /// that item: one accepted, one pending (submitted by user 2) and one
    /// rejected. Only the accepted one is visible to everybody; user 2 also
    /// sees their own pending submission.
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
            DEFINE FIELD known_members ON tags TYPE int DEFAULT 0 PERMISSIONS FULL;

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

            DEFINE TABLE workshop_item_properties TYPE RELATION IN workshop_items OUT properties \
             SCHEMAFULL PERMISSIONS NONE;
            DEFINE FIELD note ON workshop_item_properties TYPE none | string PERMISSIONS FULL;
            DEFINE FIELD source ON workshop_item_properties TYPE 'system' | record<users> \
             PERMISSIONS FULL;
            DEFINE FIELD status ON workshop_item_properties TYPE -1 | 0 | 1 DEFAULT 0 PERMISSIONS \
             FULL;
            DEFINE FIELD upvote_count ON workshop_item_properties TYPE int DEFAULT 0 PERMISSIONS \
             FULL;
            DEFINE FIELD vote_count ON workshop_item_properties TYPE int DEFAULT 0 PERMISSIONS \
             FULL;

            DEFINE TABLE users TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
            DEFINE FIELD id ON users TYPE int PERMISSIONS FULL;

            DEFINE TABLE votes TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
            DEFINE FIELD id ON votes TYPE { item: record<workshop_items>, link: \
             record<properties>, user: record<users> } PERMISSIONS FULL;
            DEFINE FIELD score ON votes TYPE int PERMISSIONS FULL;
            DEFINE FIELD user ON votes TYPE record<users> PERMISSIONS FULL;
            DEFINE FIELD when ON votes TYPE datetime PERMISSIONS FULL;

            CREATE apps:1 SET id = 1, name = 'Test App';
            CREATE usernames:1 SET id = 1, name = 'Test Author';
            CREATE tags:test SET id = 'test', display_name = 'Test Tag';
            CREATE users:1 SET id = 1;
            CREATE users:2 SET id = 2;

            CREATE workshop_items:100 SET id = 100, app = apps:1, author = usernames:1, \
             description = 'item 100', languages = [], last_updated = 0, score = 1.0f, tags = \
             [tags:test], title = 'Item 100';

            CREATE properties:{ class: 'Type', value: 'accepted' };
            CREATE properties:{ class: 'Type', value: 'pending' };
            CREATE properties:{ class: 'Type', value: 'rejected' };

            RELATE workshop_items:100 -> workshop_item_properties -> properties:{ class: 'Type', \
             value: 'accepted' } SET status = 1, source = 'system';
            RELATE workshop_items:100 -> workshop_item_properties -> properties:{ class: 'Type', \
             value: 'pending' } SET status = 0, source = users:2;
            RELATE workshop_items:100 -> workshop_item_properties -> properties:{ class: 'Type', \
             value: 'rejected' } SET status = -1, source = 'system';
            ",
        )
        .await
        .unwrap()
        .check()
        .unwrap();

        db
    }

    async fn list_property_values(
        db: &Surreal<Db>,
        user: Option<IUserID>,
    ) -> Vec<(String, Status)> {
        let items = query_inner(
            1,
            0,
            100,
            None,
            vec![],
            None,
            None,
            None,
            None,
            vec![],
            vec![],
            db,
            user,
        )
        .await
        .expect("query should succeed");
        let item = items.first().expect("item 100 should be returned");
        let mut props: Vec<(String, Status)> = item
            .properties
            .iter()
            .map(|prop| {
                assert_eq!(prop.out.class, Class::Type);
                (prop.out.value.clone(), prop.status)
            })
            .collect();
        props.sort();
        props
    }

    /// Regression test: the list query must only return accepted properties for
    /// an anonymous caller. Pending and rejected edges leaked through when the
    /// status filter was attached to the projection's alias instead of its
    /// expression, so the filter was parsed but never evaluated.
    #[tokio::test]
    async fn anonymous_listing_only_returns_accepted_properties() {
        let db = seed_db().await;

        assert_eq!(
            list_property_values(&db, None).await,
            vec![("accepted".to_string(), Status::Accepted)],
            "anonymous callers must not see pending or rejected properties"
        );
    }

    /// A signed-in user sees accepted properties plus their own submissions,
    /// whatever the status, and still never sees somebody else's pending work.
    #[tokio::test]
    async fn user_listing_returns_accepted_properties_and_own_submissions() {
        let db = seed_db().await;

        assert_eq!(
            list_property_values(&db, Some(IUserID::from(2i64))).await,
            vec![
                ("accepted".to_string(), Status::Accepted),
                ("pending".to_string(), Status::Pending),
            ],
            "user 2 should see the accepted property and their own pending one"
        );

        let other_user = list_property_values(&db, Some(IUserID::from(1i64))).await;
        assert_eq!(
            other_user,
            vec![("accepted".to_string(), Status::Accepted)],
            "user 1 submitted nothing, so they only see the accepted property"
        );
        assert!(
            !other_user
                .iter()
                .any(|(value, _)| value == "pending" || value == "rejected"),
            "user 1 must not see user 2's pending submission"
        );
    }

    /// Adds two more items to the seeded app so the three items have distinct
    /// `last_updated` stamps: 0, 100 and 200.
    async fn seed_more_timestamps(db: &Surreal<Db>) {
        db.query(
            "
            CREATE workshop_items:200 SET id = 200, app = apps:1, author = usernames:1, \
             description = 'item 200', languages = [], last_updated = 100, score = 1.0f, tags = \
             [], title = 'Item 200';
            CREATE workshop_items:300 SET id = 300, app = apps:1, author = usernames:1, \
             description = 'item 300', languages = [], last_updated = 200, score = 1.0f, tags = \
             [], title = 'Item 300';
            ",
        )
        .await
        .unwrap()
        .check()
        .unwrap();
    }

    /// The `last_updated` stamps that the list query returns for the given
    /// bounds, in ascending order.
    async fn list_last_updated(
        db: &Surreal<Db>,
        updated_before: Option<i64>,
        updated_after: Option<i64>,
    ) -> Vec<u64> {
        let items = query_inner(
            1,
            0,
            100,
            None,
            vec![],
            None,
            updated_before,
            updated_after,
            None,
            vec![],
            vec![],
            db,
            None,
        )
        .await
        .expect("query should succeed");
        let mut stamps: Vec<u64> = items.into_iter().map(|item| item.last_updated).collect();
        stamps.sort_unstable();
        stamps
    }

    /// `updated_before` and `updated_after` must bound `last_updated` on the
    /// side their names say: `before` keeps the older items, `after` keeps the
    /// newer ones.
    #[tokio::test]
    async fn updated_bounds_filter_on_the_correct_side() {
        let db = seed_db().await;
        seed_more_timestamps(&db).await;

        assert_eq!(
            list_last_updated(&db, None, None).await,
            vec![0, 100, 200],
            "with no bounds the query returns every item"
        );

        assert_eq!(
            list_last_updated(&db, Some(150), None).await,
            vec![0, 100],
            "updated_before=150 keeps only the items updated before 150"
        );

        assert_eq!(
            list_last_updated(&db, None, Some(150)).await,
            vec![200],
            "updated_after=150 keeps only the items updated after 150"
        );

        assert_eq!(
            list_last_updated(&db, Some(150), Some(50)).await,
            vec![100],
            "both bounds together keep the items inside the window"
        );

        assert!(
            list_last_updated(&db, Some(50), Some(150)).await.is_empty(),
            "an inverted window matches nothing"
        );
    }

    /// The property `source` must survive the projection so the UI can tell
    /// which entries the caller submitted themselves.
    #[tokio::test]
    async fn property_source_is_projected() {
        let db = seed_db().await;

        let items = query_inner(
            1,
            0,
            100,
            None,
            vec![],
            None,
            None,
            None,
            None,
            vec![],
            vec![],
            &db,
            Some(IUserID::from(2i64)),
        )
        .await
        .expect("query should succeed");
        let item = items.first().expect("item 100 should be returned");
        let pending = item
            .properties
            .iter()
            .find(|prop| prop.out.value == "pending")
            .expect("user 2 sees their own pending property");
        assert!(
            matches!(&pending.source, ExternalSource::User(id) if *id == 2i64.into()),
            "the pending property was submitted by user 2"
        );
    }

    /// Adds a second tag, plus three items, so a tag filter has something to
    /// discriminate on. Item 100 from `seed_db` already carries `tags:test`.
    /// Item 600 points at `tags:deleted`, a link with no row behind it.
    async fn seed_more_tags(db: &Surreal<Db>) {
        db.query(
            "
            CREATE tags:other SET id = 'other', display_name = 'Other Tag';

            CREATE workshop_items:400 SET id = 400, app = apps:1, author = usernames:1, \
             description = 'item 400', languages = [], last_updated = 0, score = 1.0f, tags = \
             [tags:test, tags:other], title = 'Item 400';
            CREATE workshop_items:500 SET id = 500, app = apps:1, author = usernames:1, \
             description = 'item 500', languages = [], last_updated = 0, score = 1.0f, tags = \
             [tags:other], title = 'Item 500';
            CREATE workshop_items:600 SET id = 600, app = apps:1, author = usernames:1, \
             description = 'item 600', languages = [], last_updated = 0, score = 1.0f, tags = \
             [tags:test, tags:deleted], title = 'Item 600';
            ",
        )
        .await
        .unwrap()
        .check()
        .unwrap();
    }

    /// The item ids the list query returns for the given tag filter, ascending.
    async fn list_ids_for_tags(db: &Surreal<Db>, tags: &[&str]) -> Vec<i64> {
        let items = query_inner(
            1,
            0,
            100,
            None,
            tags.iter().map(|tag| (*tag).to_string()).collect(),
            None,
            None,
            None,
            None,
            vec![],
            vec![],
            db,
            None,
        )
        .await
        .expect("query should succeed");
        let mut ids: Vec<i64> = items.into_iter().map(|item| item.id.into()).collect();
        ids.sort_unstable();
        ids
    }

    /// `CONTAINSALL` matches on the stored record links, and it keeps only the
    /// items that carry every tag asked for.
    #[tokio::test]
    async fn tag_filter_keeps_the_items_that_carry_every_tag() {
        let db = seed_db().await;
        seed_more_tags(&db).await;

        assert_eq!(
            list_ids_for_tags(&db, &[]).await,
            vec![100, 400, 500, 600],
            "no tag filter returns every item"
        );
        assert_eq!(
            list_ids_for_tags(&db, &["test"]).await,
            vec![100, 400, 600],
            "one tag keeps every item that carries it"
        );
        assert_eq!(list_ids_for_tags(&db, &["other"]).await, vec![400, 500]);
        assert_eq!(
            list_ids_for_tags(&db, &["test", "other"]).await,
            vec![400],
            "two tags keep only the item that carries both"
        );
        assert!(
            list_ids_for_tags(&db, &["absent"]).await.is_empty(),
            "a tag no item carries matches nothing"
        );
    }

    /// The `tags.filter(|$tag| $tag.exists()).*` projection does not change
    /// what `CONTAINSALL` matches. SurrealDB checks the WHERE clause against
    /// the stored document and only then plucks the output, so the expansion
    /// cannot reach the filter. Each projection shape below matches the same
    /// item.
    #[tokio::test]
    async fn the_tag_projection_does_not_change_what_containsall_matches() {
        /// Runs the tag filter with the given extra projection fields spliced
        /// in, and returns the ids that survived, ascending.
        async fn ids_for_projection(db: &Surreal<Db>, projection: &str) -> Vec<i64> {
            let sql = format!(
                "SELECT VALUE record::id(id) FROM (SELECT *{projection} FROM workshop_items WHERE \
                 app = apps:1 AND tags CONTAINSALL [tags:test, tags:other]);"
            );
            let mut response = db.query(sql).await.unwrap().check().unwrap();
            let mut ids: Vec<i64> = response.take(0).unwrap();
            ids.sort_unstable();
            ids
        }

        let db = seed_db().await;
        seed_more_tags(&db).await;

        assert_eq!(
            ids_for_projection(&db, "").await,
            vec![400],
            "no expansion at all"
        );
        assert_eq!(
            ids_for_projection(&db, ", tags.filter(|$tag| $tag.exists()).*").await,
            vec![400],
            "the expansion the list query uses"
        );
        assert_eq!(
            ids_for_projection(&db, ", tags.*").await,
            vec![400],
            "a plain expansion, without the exists() filter"
        );
    }

    /// A tag link whose row was deleted still filters, because the WHERE clause
    /// sees the raw link. The projection drops it, because
    /// `tags.filter(|$tag| $tag.exists())` cannot expand it.
    #[tokio::test]
    async fn a_dangling_tag_link_filters_but_is_not_projected() {
        let db = seed_db().await;
        seed_more_tags(&db).await;

        assert_eq!(
            list_ids_for_tags(&db, &["deleted"]).await,
            vec![600],
            "item 600 carries tags:deleted, which has no row"
        );

        let items = query_inner(
            1,
            0,
            100,
            None,
            vec!["deleted".to_string()],
            None,
            None,
            None,
            None,
            vec![],
            vec![],
            &db,
            None,
        )
        .await
        .expect("query should succeed");
        let item = items.first().expect("item 600 should be returned");
        assert_eq!(
            item.tags
                .iter()
                .map(|tag| String::from(tag.id.clone()))
                .collect::<Vec<_>>(),
            vec!["test".to_string()],
            "the dangling link is dropped from the projected tags"
        );
    }

    /// Steam tag keys are not identifiers: production carries `tags:`1.6``
    /// and `tags:`Clothing/Armor`` (see `v2_exported_for_v3.surql`). The
    /// filter builds a record id literal for each one, so the keys must
    /// survive `to_sql` quoting and still match.
    #[tokio::test]
    async fn tag_filter_handles_keys_that_are_not_identifiers() {
        let db = seed_db().await;
        db.query(
            "
            CREATE tags:⟨1.6⟩ SET id = '1.6', display_name = '1.6';
            CREATE tags:⟨Clothing/Armor⟩ SET id = 'Clothing/Armor', display_name = \
             'Clothing/Armor';

            CREATE workshop_items:700 SET id = 700, app = apps:1, author = usernames:1, \
             description = 'item 700', languages = [], last_updated = 0, score = 1.0f, tags = \
             [tags:⟨1.6⟩, tags:⟨Clothing/Armor⟩], title = 'Item 700';
            ",
        )
        .await
        .unwrap()
        .check()
        .unwrap();

        assert_eq!(list_ids_for_tags(&db, &["1.6"]).await, vec![700]);
        assert_eq!(list_ids_for_tags(&db, &["Clothing/Armor"]).await, vec![700]);
        assert_eq!(
            list_ids_for_tags(&db, &["1.6", "Clothing/Armor"]).await,
            vec![700]
        );
        assert!(
            list_ids_for_tags(&db, &["1.6", "test"]).await.is_empty(),
            "item 700 does not carry tags:test, item 100 does not carry tags:⟨1.6⟩"
        );
    }
}
