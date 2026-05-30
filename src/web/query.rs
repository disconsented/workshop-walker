use salvo::{
    oapi::{endpoint, extract::QueryParam}, prelude::Json,
    Request,
    Writer,
};
use snafu::{ResultExt, Whatever};
use surrealdb::{engine::local::Db, Surreal};
use surrealdb_core::sql::{
    field::Selector, statements::SelectStatement, Expr, Field, Fields, Idiom, Limit, Start,
};
use surrealdb_types::{SurrealValue, Table, ToSql};
use tracing::{debug, info_span, instrument, Instrument};

use crate::{
    db::model::{ExternalWorkshopItem, InternalWorkshopItem, OrderBy},
    processing::language_actor::DetectedLanguage,
    web,
    web::DB_POOL,
};

#[instrument(skip_all)]
#[endpoint]
pub async fn list(
    _: &mut Request,
    app: QueryParam<i64, true>,
    page: QueryParam<u64, false>,
    limit: QueryParam<u64, false>,
    languages: QueryParam<DetectedLanguage, false>,
    mut tags: QueryParam<Vec<String>, false>,
    mut title: QueryParam<String, false>,
    last_updated: QueryParam<u64, false>,
    mut order_by: QueryParam<OrderBy, false>,
) -> web::Result<Json<Vec<ExternalWorkshopItem>>> {
    let page = page.unwrap_or(0);
    let limit = limit.unwrap_or(100).min(100);
    let db: &Surreal<Db> = DB_POOL.get().expect("Getting db connection");
    #[instrument(skip_all)]
    async fn query(
        _app: i64,
        page: u64,
        limit: u64,
        _languages: Option<DetectedLanguage>,
        _tags: Vec<String>,
        _title: Option<String>,
        _last_updated: Option<u64>,
        _order_by: Option<OrderBy>,
        db: &Surreal<Db>,
    ) -> web::Result<Vec<ExternalWorkshopItem>, Whatever> {
        let mut stmt = SelectStatement::default();
        {
            // stmt.fields = Fields::Select(vec![
            //     Field::All,
            //     Field::Single(Selector {
            //         expr: Expr::Idiom(Idiom::field("appid".into())),
            //         alias: Some(Idiom::field("app".into())),
            //     }),
            // ]);

            //     {
            //         stmt.expr.0.push(Field::Single {
            //             expr: idiom(
            //                 "tags.{id: id.to_string(), app,
            // display_name}",
            //             )
            //             .expect(
            //                 "expanding tags
            // idiom",
            //             )
            //             .into(),
            //             alias: Some("tags".into()),
            //         });
            //     }
            //     {
            //         stmt.expr.0.push(Field::Single {
            //             // Select _approved_ props only
            //             expr: idiom(
            //
            // r"->workshop_item_properties.filter(|$prop|$prop.status ==
            // 1)[*].{                                 id:
            // id.to_string(),                                 in:
            // in.to_string(),                                 out:
            // out.id.{                                     class,
            //                                     `value`
            //                                 },
            //                                 source: 'system',
            //                                 status,
            //                                 upvote_count,
            //                                 vote_count
            //                             }",
            //             )
            //             .expect("expanding properties idiom")
            //             .into(),
            //             alias: Some("properties".into()),
            //         });
            //     }
            //     if let Some(OrderBy::Dependents) = order_by {
            //         stmt.expr.0.push(Field::Single {
            //             expr: idiom(" <-item_dependencies.len()")
            //                 .expect("expanding item_tags idiom")
            //                 .into(),
            //             alias: Some("dependencies_length".into()),
            //         });
            //     }
        }
        //
        // stmt.limit = Some(Limit(Expr::from_public_value(limit.into_value())));
        // stmt.start = Some(Start(Expr::from_public_value((page * limit).into_value())));
        // stmt.what.push(Expr::from_public_value(
        //     Table::new("workshop_items").into_value(),
        // ));

        // stmt.cond = {
        //     let conditions = vec![
        //         languages.map(|lang| {
        //             Expression::new(
        //                 Value::Array(vec![(lang as u8).into(),
        // Value::Number(0.into())].into()),
        // Operator::ContainAny,
        // Value::Idiom("languages".into()),             )
        //         }),
        //         last_updated.map(|updated| {
        //             Expression::new(
        //                 Value::Idiom("last_updated".into()),
        //                 Operator::MoreThanOrEqual,
        //                 Value::Number(updated.into()),
        //             )
        //         }),
        //         (!tags.is_empty()).then(|| {
        //             if true {
        //                 // All
        //                 Expression::new(
        //                     Value::Idiom("tags".into()),
        //                     Operator::ContainAll,
        //                     Value::Array(
        //                         tags.iter()
        //                             .map(|tag| {
        //                                 to_value(
        //                                     RecordId::from_str(tag)
        //                                         .map(ITagID::from)
        //
        // .unwrap_or(ITagID::from(tag.to_string())),
        // )                                 .unwrap()
        //                             })
        //                             .collect::<Vec<_>>()
        //                             .into(),
        //                     ),
        //                 )
        //             } else {
        //                 // Either (unsupported for now)
        //                 Expression::new(
        //                     Value::Idiom(
        //                         idiom(&format!(
        //                             "tags.any(|$var| {} )",
        //                             tags.into_iter()
        //                                 .map(|tag| format!(
        //                                     "$var.id == {}",
        //                                     RecordId::from_str(&tag)
        //                                         .unwrap_or(RecordId::new("tags",
        // tag))                                 ))
        //                                 .join(" OR ")
        //                         ))
        //                         .unwrap(),
        //                     ),
        //                     Operator::Equal,
        //                     Value::Bool(true),
        //                 )
        //             }
        //         }),
        //         title.map(|title_query| {
        //             Expression::new(
        //                 Value::Idiom("title".into()),
        //                 Operator::Like,
        //                 Value::Strand(title_query.into()),
        //             )
        //         }),
        //         Some(Expression::new(
        //             Value::Idiom("appid".into()),
        //             Operator::Equal,
        //             Value::Number(app.into()),
        //         )),
        //     ]
        //     .into_iter()
        //     .flatten()
        //     .collect::<Vec<Expr>>();
        //
        //     if conditions.is_empty() {
        //         None
        //     } else {
        //         let mut values = Value::None;
        //         for mut condition in &conditions.into_iter().chunks(2) {
        //             let c1 = condition.next();
        //             let c2 = condition.next();
        //             match (values, c1, c2) {
        //                 (Value::None, Some(expr1), Some(expr2)) => {
        //                     values = Value::Expression(Box::from(Expression::new(
        //                         expr1.into(),
        //                         Operator::And,
        //                         expr2.into(),
        //                     )));
        //                 }
        //                 (Value::None, Some(expr1), None) => {
        //                     values = Value::Expression(Box::from(expr1));
        //                 }
        //                 (Value::Expression(old), Some(expr1), Some(expr2)) => {
        //                     values = Value::Expression(Box::from(Expression::new(
        //                         Value::Expression(old),
        //                         Operator::And,
        //                         Value::Expression(Box::from(Expression::new(
        //                             expr1.into(),
        //                             Operator::And,
        //                             expr2.into(),
        //                         ))),
        //                     )));
        //                 }
        //                 (Value::Expression(old), Some(expr1), None) => {
        //                     values = Value::Expression(Box::from(Expression::new(
        //                         Value::Expression(old),
        //                         Operator::And,
        //                         expr1.into(),
        //                     )));
        //                 }
        //                 (other, ..) => {
        //                     values = other;
        //                 }
        //             }
        //         }
        //         let mut cond = Cond::default();
        //         cond.0 = to_value(values).unwrap();
        //         Some(cond)
        //     }
        // };

        // // A horrendous hack for ordering, because, the types are not xposed.
        // stmt.order = order_by.map(|order_term| {
        //     use serde_json::{Map, Value};
        //     use str_macro::str;
        //     let terms = Map::from_iter([
        //         (
        //             str!("value"),
        //
        // serde_json::to_value(idiom(order_term.column_name()).unwrap()).unwrap(),
        //         ),
        //         (str!("collate"), Value::Bool(false)),
        //         (str!("numeric"), Value::Bool(false)),
        //         (str!("direction"), Value::Bool(false)),
        //     ]);
        //     serde_json::from_value(Value::Object(Map::from_iter([(
        //         str!("Order"),
        //         Value::Array(vec![Value::Object(terms)]),
        //     )])))
        //     .unwrap()
        // });

        let stmt = r#"SELECT
    *,
    ->workshop_item_properties AS properties,
    tags.{id, app, display_name}
FROM workshop_items
LIMIT 50
START 0;
"#;
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
