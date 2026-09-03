use salvo::{
    Depot, Request, Writer,
    oapi::{endpoint, extract::QueryParam},
    prelude::Json,
};
use snafu::{ResultExt, Whatever};
use surrealdb::{Surreal, engine::local::Db};
use surrealdb_core::sql::{
    BinaryOperator, Closure, Cond, Dir, Expr, Field, Fields, Idiom, Kind, Limit, Literal, Lookup,
    Order, Param, Part, RecordIdKeyLit, RecordIdLit, Start,
    field::Selector,
    literal::ObjectEntry,
    lookup::{LookupKind, LookupSubject},
    order::{OrderList, Ordering},
    part::DestructurePart,
    statements::SelectStatement,
};
use surrealdb_types::{RecordId, SurrealValue, ToSql};
use tracing::{Instrument, debug, info_span, instrument, trace};

use crate::{
    db::{
        IAppID, ITagID, IUserID,
        model::{ExternalWorkshopItem, InternalWorkshopItem, OrderBy, Status},
    },
    processing::language_actor::DetectedLanguage,
    web,
    web::{DB_POOL, auth},
};

// ToDo: Seperate out filtering to its own struct
// And, handle pagination based on the last element for performance
#[instrument(skip_all)]
#[endpoint]
pub async fn list(
    req: &mut Request,
    depot: &mut Depot,
    app: QueryParam<i64, true>,
    page: QueryParam<u64, false>,
    limit: QueryParam<u64, false>,
    language: QueryParam<DetectedLanguage, false>,
    mut tags: QueryParam<Vec<String>, false>,
    mut title: QueryParam<String, false>,
    last_updated: QueryParam<i64, false>,
    mut order_by: QueryParam<OrderBy, false>,
) -> web::Result<Json<Vec<ExternalWorkshopItem>>> {
    let page = page.unwrap_or(0);
    let limit = limit.unwrap_or(100).min(100);
    let db: &Surreal<Db> = DB_POOL.get().expect("Getting db connection");
    let user = auth::get_user_from_depot(depot).map(Into::into);

    let results = query_inner(
        app.into_inner(),
        page,
        limit,
        *language,
        tags.take().unwrap_or_default(),
        title.take(),
        *last_updated,
        order_by.take(),
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
    last_updated: Option<i64>,
    order_by: Option<OrderBy>,
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
    // `item::get_item`: looks up the per-user vote record keyed the same way the
    // write side keys it (see `properties_repository::vote`): votes:{ link, user,
    // item }. `.score` on a record id that doesn't exist yields NONE, so
    // un-voted properties come back as `None`. Only computable with a user.
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
    let mut stmt = SelectStatement::default();
    stmt.what = vec![Expr::Table("workshop_items".into())];
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
                        cond: Some(Cond(properties_condition)),
                        ..Default::default()
                    })),
                    Part::Destructure(prop_fields.to_vec()),
                ])),
                alias: Some(Idiom::field("properties".to_string())),
            }),
            // Author's are more so considered eventually consistent
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
            conditions.push(Expr::Binary {
                left: Box::new(Expr::Idiom(Idiom::field("languages".to_string()))),
                op: BinaryOperator::ContainAny,
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

        if let Some(title) = title {
            conditions.push(Expr::Binary {
                left: Box::new(Expr::Idiom(Idiom::field("title".to_string()))),
                op: BinaryOperator::Contain,
                right: Box::new(Expr::Literal(Literal::String(title.into()))),
            });
        }

        if let Some(last_updated) = last_updated {
            conditions.push(Expr::Binary {
                left: Box::new(Expr::Idiom(Idiom::field("last_updated".to_string()))),
                op: BinaryOperator::MoreThan,
                right: Box::new(Expr::Literal(Literal::Integer(last_updated))),
            });
        }

        let first = conditions
            .pop()
            .expect("Expected at least one condition to be present");
        if conditions.len() == 1 {
            Some(Cond(first))
        } else {
            Some(Cond(conditions.into_iter().fold(first, |old, next| {
                Expr::Binary {
                    left: Box::new(old),
                    op: BinaryOperator::And,
                    right: Box::new(next),
                }
            })))
        }
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
        let items = query_inner(1, 0, 100, None, vec![], None, None, None, db, user)
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
}
