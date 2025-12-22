use std::str::FromStr;

use itertools::Itertools;
use salvo::{
    Request, Writer,
    oapi::{endpoint, extract::QueryParam},
    prelude::Json,
};
use snafu::{ResultExt, Whatever};
use surrealdb::{
    RecordId, Surreal,
    engine::local::Db,
    sql::{
        Cond, Expression, Field, Limit, Operator, Start, Value, idiom, statements::SelectStatement,
        to_value,
    },
};
use tracing::{Instrument, info, info_span, instrument};

use crate::{
    db::model::{OrderBy, WorkshopItem},
    processing::language_actor::DetectedLanguage,
    web,
    web::DB_POOL,
};
#[instrument(skip_all)]
#[endpoint]
pub async fn list(
    _: &mut Request,
    app: QueryParam<u64, true>,
    page: QueryParam<u64, false>,
    limit: QueryParam<u64, false>,
    languages: QueryParam<DetectedLanguage, false>,
    mut tags: QueryParam<Vec<String>, false>,
    mut title: QueryParam<String, false>,
    last_updated: QueryParam<u64, false>,
    mut order_by: QueryParam<OrderBy, false>,
) -> web::Result<Json<Vec<WorkshopItem<String>>>> {
    let page = page.unwrap_or(0);
    let limit = limit.unwrap_or(100).min(100);
    let db: &Surreal<Db> = DB_POOL.get().expect("Getting db connection");
    #[instrument(skip_all)]
    async fn query(
        app: u64,
        page: u64,
        limit: u64,
        languages: Option<DetectedLanguage>,
        tags: Vec<String>,
        title: Option<String>,
        last_updated: Option<u64>,
        order_by: Option<OrderBy>,
        db: &Surreal<Db>,
    ) -> web::Result<Vec<WorkshopItem<String>>, Whatever> {
        let mut stmt = SelectStatement::default();
        {
            stmt.expr.0.append(&mut vec![Field::All]);

            {
                stmt.expr.0.push(Field::Single {
                    expr: idiom("tags.{id: id.to_string(), app_id, display_name}")
                        .expect("expanding tags idiom")
                        .into(),
                    alias: Some("tags".into()),
                });
            }
            {
                stmt.expr.0.push(Field::Single {
                    // Select _approved_ props only
                    expr: idiom(
                        r"->workshop_item_properties.filter(|$prop|$prop.status == 1)[*].{
                                        id: id.to_string(),
                                        in: in.to_string(),
                                        out: out.id.{
                                            class,
                                            `value`
                                        },
                                        source: 'system',
                                        status,
                                        upvote_count,
                                        vote_count
                                    }",
                    )
                    .expect("expanding properties idiom")
                    .into(),
                    alias: Some("properties".into()),
                });
            }
            if let Some(OrderBy::Dependents) = order_by {
                stmt.expr.0.push(Field::Single {
                    expr: idiom(" <-item_dependencies.len()")
                        .expect("expanding item_tags idiom")
                        .into(),
                    alias: Some("dependencies_length".into()),
                });
            }
        }

        stmt.limit = Some({
            let mut d = Limit::default();
            d.0 = to_value(limit).whatever_context("limit")?;
            d
        });
        stmt.start = Some({
            let mut s = Start::default();
            s.0 = to_value(limit * page).whatever_context("start limit")?;
            s
        });

        stmt.parallel = true;
        stmt.what.0.push(Value::Table("workshop_items".into()));
        stmt.cond = {
            let conditions = vec![
                languages.map(|lang| {
                    Expression::new(
                        Value::Array(vec![(lang as u8).into(), Value::Number(0.into())].into()),
                        Operator::ContainAny,
                        Value::Idiom("languages".into()),
                    )
                }),
                last_updated.map(|updated| {
                    Expression::new(
                        Value::Idiom("last_updated".into()),
                        Operator::MoreThanOrEqual,
                        Value::Number(updated.into()),
                    )
                }),
                (!tags.is_empty()).then(|| {
                    if true {
                        // All
                        Expression::new(
                            Value::Idiom("tags".into()),
                            Operator::ContainAll,
                            Value::Array(
                                tags.iter()
                                    .map(|tag| {
                                        to_value(
                                            RecordId::from_str(tag)
                                                .unwrap_or(RecordId::from_table_key("tags", tag)),
                                        )
                                        .unwrap()
                                    })
                                    .collect::<Vec<_>>()
                                    .into(),
                            ),
                        )
                    } else {
                        // Either (unsupported for now)
                        Expression::new(
                            Value::Idiom(
                                idiom(&format!(
                                    "tags.any(|$var| {} )",
                                    tags.into_iter()
                                        .map(|tag| format!(
                                            "$var.id == {}",
                                            RecordId::from_str(&tag)
                                                .unwrap_or(RecordId::from_table_key("tags", tag))
                                        ))
                                        .join(" OR ")
                                ))
                                .unwrap(),
                            ),
                            Operator::Equal,
                            Value::Bool(true),
                        )
                    }
                }),
                title.map(|title_query| {
                    Expression::new(
                        Value::Idiom("title".into()),
                        Operator::Like,
                        Value::Strand(title_query.into()),
                    )
                }),
                Some(Expression::new(
                    Value::Idiom("appid".into()),
                    Operator::Equal,
                    Value::Number(app.into()),
                )),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<Expression>>();

            if conditions.is_empty() {
                None
            } else {
                let mut values = Value::None;
                for mut condition in &conditions.into_iter().chunks(2) {
                    let c1 = condition.next();
                    let c2 = condition.next();
                    match (values, c1, c2) {
                        (Value::None, Some(expr1), Some(expr2)) => {
                            values = Value::Expression(Box::from(Expression::new(
                                expr1.into(),
                                Operator::And,
                                expr2.into(),
                            )));
                        }
                        (Value::None, Some(expr1), None) => {
                            values = Value::Expression(Box::from(expr1));
                        }
                        (Value::Expression(old), Some(expr1), Some(expr2)) => {
                            values = Value::Expression(Box::from(Expression::new(
                                Value::Expression(old),
                                Operator::And,
                                Value::Expression(Box::from(Expression::new(
                                    expr1.into(),
                                    Operator::And,
                                    expr2.into(),
                                ))),
                            )));
                        }
                        (Value::Expression(old), Some(expr1), None) => {
                            values = Value::Expression(Box::from(Expression::new(
                                Value::Expression(old),
                                Operator::And,
                                expr1.into(),
                            )));
                        }
                        (other, ..) => {
                            values = other;
                        }
                    }
                }
                let mut cond = Cond::default();
                cond.0 = to_value(values).unwrap();
                Some(cond)
            }
        };

        // A horrendous hack for ordering, because, the types are not exposed.
        stmt.order = order_by.map(|order_term| {
            use serde_json::{Map, Value};
            use str_macro::str;
            let terms = Map::from_iter([
                (
                    str!("value"),
                    serde_json::to_value(idiom(order_term.column_name()).unwrap()).unwrap(),
                ),
                (str!("collate"), Value::Bool(false)),
                (str!("numeric"), Value::Bool(false)),
                (str!("direction"), Value::Bool(false)),
            ]);
            serde_json::from_value(Value::Object(Map::from_iter([(
                str!("Order"),
                Value::Array(vec![Value::Object(terms)]),
            )])))
            .unwrap()
        });

        stmt.parallel = true;

        info!("{stmt}");
        let mut results = db.query(stmt).await.whatever_context("querying")?;

        let results: Vec<WorkshopItem<RecordId>> =
            results.take(0).whatever_context("taking result")?;

        Ok(results
            .into_iter()
            .map(|res| WorkshopItem {
                appid: res.appid,
                author: res.author,
                description: res.description,
                id: res.id.key().to_string().replace("⟩", "").replace("⟨", ""),
                languages: res.languages,
                title: res.title,
                preview_url: res.preview_url,
                last_updated: res.last_updated,
                tags: res.tags,
                score: res.score,
                properties: res.properties,
            })
            .collect())
    }
    let results = query(
        app.into_inner(),
        page,
        limit,
        *languages,
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
