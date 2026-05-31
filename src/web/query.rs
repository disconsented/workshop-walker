use salvo::{
    oapi::{endpoint, extract::QueryParam}, prelude::Json,
    Request,
    Writer,
};
use snafu::{ResultExt, Whatever};
use surrealdb::{engine::local::Db, Surreal};
use surrealdb_core::sql::{
    field::Selector, lookup::{LookupKind, LookupSubject}, order::{OrderList, Ordering}, part::DestructurePart, statements::SelectStatement, BinaryOperator, Closure, Cond, Dir, Expr, Field, Fields,
    Idiom, Kind, Limit, Literal, Lookup,
    Order,
    Param,
    Part,
    RecordIdLit,
    Start,
};
use surrealdb_types::{RecordId, SurrealValue, Table, ToSql, Value};
use tracing::{debug, info_span, instrument, Instrument};

use crate::{
    db::{
        model::{ExternalWorkshopItem, InternalWorkshopItem, OrderBy, Status}, IAppID,
        ITagID,
    },
    processing::language_actor::DetectedLanguage,
    web,
    web::DB_POOL,
};

// ToDo: Seperate out filtering to its own struct
// And, handle pagination based on the last element for performance
#[instrument(skip_all)]
#[endpoint]
pub async fn list(
    _: &mut Request,
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
    #[instrument(skip_all)]
    async fn query(
        app: i64,
        page: u64,
        limit: u64,
        language: Option<DetectedLanguage>,
        tags: Vec<String>,
        title: Option<String>,
        last_updated: Option<i64>,
        order_by: Option<OrderBy>,
        db: &Surreal<Db>,
    ) -> web::Result<Vec<ExternalWorkshopItem>, Whatever> {
        let app = IAppID::from(app);
        let mut stmt = SelectStatement::default();
        stmt.what = vec![Expr::Table("workshop_items".into())];
        {
            stmt.fields = Fields::Select(vec![
                Field::All,
                Field::Single(Selector {
                    expr: Expr::Idiom(Idiom(vec![Part::Graph(Lookup {
                        kind: LookupKind::Graph(Dir::Out),
                        what: vec![LookupSubject::Table {
                            table: "workshop_item_properties".to_string(),
                            referencing_field: None,
                        }],
                        ..Default::default()
                    })])),
                    alias: Some(Idiom(vec![
                        Part::Field("properties".to_string()),
                        Part::Method(
                            "filter".to_string(),
                            vec![Expr::Closure(Box::new(Closure {
                                args: vec![(Param::new("prop".to_string()), Kind::Any)],
                                returns: None,
                                body: Expr::Binary {
                                    left: Box::new(Expr::Idiom(Idiom(vec![
                                        Part::Start(Expr::Param(Param::new("prop".to_string()))),
                                        Part::Field("status".to_string()),
                                    ]))),
                                    op: BinaryOperator::ExactEqual,
                                    right: Box::new(Expr::Literal(Literal::Integer(
                                        Status::Accepted as i64,
                                    ))),
                                },
                            }))],
                        ),
                    ])),
                }),
                Field::Single(Selector {
                    expr: Expr::Idiom(Idiom(vec![Part::Field("tags".to_string()), Part::All])),
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
                    left: Box::new(Expr::Idiom(Idiom::field("language".to_string()))),
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
                    right: Box::new(Expr::Literal(Literal::String(title))),
                });
            }

            if let Some(last_updated) = last_updated {
                conditions.push(Expr::Binary {
                    left: Box::new(Expr::Idiom(Idiom::field("last_updated".to_string()))),
                    op: BinaryOperator::MoreThan,
                    right: Box::new(Expr::Literal(Literal::Integer(last_updated))),
                });
            }

            if conditions.len() == 1 {
                Some(Cond(conditions.pop().unwrap()))
            } else {
                let first = conditions.pop().unwrap();
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

        // db.query(
        //     "SELECT *, ->workshop_item_properties AS
        // properties.filter(|$prop|$prop.status == \      1).*, tags.* FROM
        // workshop_items WHERE [app = (apps:294100)] ORDER BY last_updated \
        //      DESC LIMIT 50 START 0",
        // )
        // .await;
        debug!(sql = stmt.to_sql(), "running big query");
        let mut results = db.query(stmt).await.whatever_context("querying")?;

        let results: Vec<InternalWorkshopItem> =
            results.take(0).whatever_context("taking result")?;

        results
            .into_iter()
            .map(ExternalWorkshopItem::try_from)
            .collect::<Result<_, _>>()
            .whatever_context("converting internal to external")
    }
    let results = query(
        app.into_inner(),
        page,
        limit,
        *language,
        tags.take().unwrap_or_default(),
        title.take(),
        *last_updated,
        order_by.take(),
        db,
    )
    .instrument(info_span!("query list").or_current())
    .await?;

    Ok(Json(results))
}
