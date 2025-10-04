mod admin;
mod companions;
mod properties;

use std::{
    str::FromStr,
    sync::{Arc, OnceLock},
};

use itertools::Itertools;
use salvo::{
    __private::tracing::info,
    Router,
    oapi::{
        Components, Operation,
        extract::{PathParam, QueryParam},
    },
    prelude::*,
};
use serde_json::Map;
use snafu::{OptionExt, ResultExt, Whatever};
use snowflake::SnowflakeIdGenerator;
use str_macro::str;
use surrealdb::{
    RecordId, Surreal,
    engine::local::Db,
    sql,
    sql::{Cond, Expression, Field, Operator, statements::SelectStatement, to_value},
    syn::idiom,
};
use tokio::sync::{Mutex, OnceCell};
use tracing::{Instrument, info_span, instrument};

use crate::{
    app_config::Config,
    auth,
    db::{
        UserID,
        model::{FullWorkshopItem, OrderBy, WorkshopItem, into_string},
    },
    processing::language_actor::DetectedLanguage,
};

/// Global
static DB_POOL: OnceCell<Surreal<Db>> = OnceCell::const_new();
static ID_GENERATOR: OnceLock<Arc<Mutex<SnowflakeIdGenerator>>> = OnceLock::new();

fn get_gen() -> Arc<Mutex<SnowflakeIdGenerator>> {
    Arc::clone(ID_GENERATOR.get_or_init(|| Arc::new(Mutex::new(SnowflakeIdGenerator::new(1, 1)))))
}
///  Start the webserver returning once it exists
pub async fn start(db: Surreal<Db>, config: Arc<Config>) {
    let db = DB_POOL.get_or_init(|| async { db }).await.clone();
    let router = Router::new().push(
        Router::with_path("api")
            .hoop(max_size(1024 * 1024))
            .push(Router::with_path("list").get(list))
            .push(
                Router::with_path("item/{id}")
                    .hoop(affix_state::inject(config.clone()))
                    .hoop(auth::validate_opt)
                    .get(get),
            )
            .push(
                Router::with_path("property")
                    .hoop(affix_state::inject(config.clone()))
                    .hoop(auth::validate)
                    .post(properties::new),
            )
            .push(
                Router::with_path("vote")
                    .hoop(affix_state::inject(config.clone()))
                    .hoop(auth::validate)
                    .hoop(affix_state::inject(db.clone()))
                    .push(
                        Router::with_path("property")
                            .post(properties::vote)
                            .delete(properties::remove),
                    ),
            )
            .push(
                Router::with_path("admin")
                    .hoop(affix_state::inject(config.clone()))
                    .hoop(auth::validate)
                    .hoop(affix_state::inject(db.clone()))
                    .hoop(auth::enforce_admin)
                    .push(
                        Router::with_path("properties")
                            .put(admin::patch_workshop_item_properties)
                            .get(admin::get_workshop_item_properties),
                    )
                    .push(
                        Router::with_path("users")
                            .get(admin::get_users)
                            .put(admin::patch_user),
                    ),
            )
            .hoop(affix_state::inject(config))
            .hoop(affix_state::inject(db))
            .push(Router::with_path("login").get(auth::redirect))
            .push(Router::with_path("verify").get(auth::verify))
            .push(Router::with_path("logout").get(auth::invalidate)),
    );
    let doc = OpenApi::new("workshop-walker", "0.0.1").merge_router(&router);
    let router = router
        .push(doc.into_router("/api-doc/openapi.json"))
        .push(SwaggerUi::new("/api-doc/openapi.json").into_router("swagger-ui"));

    let router = router.push(
        Router::with_path("{**path}").get(
            StaticDir::new(["ui/build/"])
                .include_dot_files(false)
                .auto_list(true)
                .defaults("index.html")
                .fallback("index.html"),
        ),
    );

    let service = Service::new(router).hoop(Logger::new());

    let acceptor = TcpListener::new("0.0.0.0:5800").bind().await;
    Server::new(acceptor).serve(service).await;
}

/// Type alias for our Error type
pub type Result<T, E = Error> = std::result::Result<T, E>;
/// Wrapper on a Whatever struct for Salvo
pub struct Error(Box<Whatever>);

unsafe impl Send for Error {}
impl From<Whatever> for Error {
    fn from(value: Whatever) -> Self {
        Self(Box::new(value))
    }
}

impl EndpointOutRegister for Error {
    fn register(_: &mut Components, operation: &mut Operation) {
        let code = StatusCode::INTERNAL_SERVER_ERROR;

        operation.responses.insert(
            code.as_str(),
            salvo::oapi::Response::new(code.canonical_reason().unwrap_or_default()),
        );
    }
}

#[async_trait]
impl Writer for Error {
    async fn write(mut self, _req: &mut Request, _: &mut Depot, res: &mut Response) {
        res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
        res.render(Text::Plain(format!("Error: {:#?}", self.0)));
    }
}

#[endpoint]
#[instrument]
async fn get(id: PathParam<String>, depot: &mut Depot) -> Result<Json<FullWorkshopItem>> {
    let id = id.0;
    let db: &Surreal<Db> = DB_POOL.get().expect("DB not initialised");
    let user = auth::get_user(depot);
    async fn query(
        id: String,
        db: &Surreal<Db>,
        user: Option<String>,
    ) -> Result<FullWorkshopItem, Whatever> {
        let id = RecordId::from_table_key("workshop_items", &id);
        let mut response = db
            .query(
                "SELECT in.appid as appid, in.description as description, in.id as id, in.title
                 as title, in.author as author, in.languages as languages, in.last_updated as
                 last_updated, in.score as score, in.tags.{id: id.to_string(), app_id, \
                 display_name} as tags, in.preview_url as preview_url, [] as properties FROM \
                 $id<-item_dependencies.*;",
            )
            .query(
                "SELECT out.appid as appid, out.description as description, out.id as id,
                 out.author as author, out.languages as languages, out.last_updated as
                 last_updated, out.title as title, out.score as score, out.tags.{id: \
                 id.to_string(), app_id, display_name} as tags, out.preview_url as preview_url, \
                 [] as properties
                 FROM $id->item_dependencies.*;",
            )
            .bind(("id", id.clone()))
            .await
            .whatever_context("getting dependants")?;
        let dependants: Vec<WorkshopItem<RecordId>> =
            response.take(0).whatever_context("no dependants found")?;
        let dependencies: Vec<WorkshopItem<RecordId>> =
            response.take(1).whatever_context("no dependencies found")?;

        let result = {
            let mut res = match user {
                None => db
                    .query(
                        r"SELECT *, tags.{id: id.to_string(), app_id, display_name} as tags,
                    ->workshop_item_properties.filter(|$prop|$prop.status == 1)[*].{
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
                    } as properties FROM $id",
                    )
                    .bind(("id", id))
                    .await
                    .whatever_context("getting item (unauth)")?,
                // array::filter doesn't capture scoped variables yet hence why this is being
                // formatted in
                Some(user) => db
                    .query(format!(
                        "SELECT *, tags.{{id: id.to_string(), app_id, display_name}} as tags,
                                        ->workshop_item_properties.filter(|$prop|$prop.status == 1 \
                         || $prop.source == {})[*].{{
                                            id: id.to_string(),
                                            in: in.to_string(),
                                            out: out.id.{{
                                                class,
                                                `value`
                                            }},
                                            source: 'system',
                                            status,
                                            upvote_count,
                                            vote_count,
                                            vote_state: votes:{{item: $id, link: out, user: \
                         {0}}}.score
                                        }} as properties FROM $id",
                        UserID::from(user).into_recordid()
                    ))
                    .bind(("id", id))
                    .await
                    .whatever_context("getting item (auth)")?,
            };

            let result: Option<WorkshopItem<RecordId>> =
                res.take(0).whatever_context("not found")?;
            result.whatever_context("not found")?
        };

        Ok(FullWorkshopItem {
            appid: result.appid,
            description: result.description,
            id: into_string(result.id.key()),
            title: result.title,
            preview_url: result.preview_url,
            languages: result.languages,
            author: result.author,
            last_updated: result.last_updated,
            dependencies: dependencies
                .into_iter()
                .map(|e| FullWorkshopItem {
                    appid: e.appid,
                    author: e.author,
                    dependants: vec![],
                    dependencies: vec![],
                    description: e.description,
                    id: into_string(e.id.key()),
                    languages: e.languages,
                    title: e.title,
                    preview_url: e.preview_url,
                    last_updated: e.last_updated,
                    tags: e.tags,
                    score: e.score,
                    properties: e.properties,
                })
                .collect(),

            dependants: dependants
                .into_iter()
                .map(|e| FullWorkshopItem {
                    appid: e.appid,
                    author: e.author,
                    dependants: vec![],
                    dependencies: vec![],
                    description: e.description,
                    id: into_string(e.id.key()),
                    languages: e.languages,
                    title: e.title,
                    preview_url: e.preview_url,
                    last_updated: e.last_updated,
                    tags: e.tags,
                    score: e.score,
                    properties: e.properties,
                })
                .collect(),
            tags: result.tags,
            score: result.score,
            properties: result.properties,
        })
    }
    let results = query(id, db, user).await?;

    Ok(Json(results))
}
#[instrument(skip_all)]
#[endpoint]
async fn list(
    _req: &mut Request,
    page: QueryParam<u64, false>,
    limit: QueryParam<u64, false>,
    languages: QueryParam<DetectedLanguage, false>,
    mut tags: QueryParam<Vec<String>, false>,
    mut title: QueryParam<String, false>,
    last_updated: QueryParam<u64, false>,
    mut order_by: QueryParam<OrderBy, false>,
) -> Result<Json<Vec<WorkshopItem<String>>>> {
    let page = page.unwrap_or(0);
    let limit = limit.unwrap_or(100).min(100);
    let db: &Surreal<Db> = DB_POOL.get().expect("Getting db connection");
    #[instrument(skip_all)]
    async fn query(
        page: u64,
        limit: u64,
        languages: Option<DetectedLanguage>,
        tags: Vec<String>,
        title: Option<String>,
        last_updated: Option<u64>,
        order_by: Option<OrderBy>,
        db: &Surreal<Db>,
    ) -> Result<Vec<WorkshopItem<String>>, Whatever> {
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
            let mut d = surrealdb::sql::Limit::default();
            d.0 = to_value(limit).whatever_context("limit")?;
            d
        });
        stmt.start = Some({
            let mut s = surrealdb::sql::Start::default();
            s.0 = to_value(limit * page).whatever_context("start limit")?;
            s
        });

        stmt.parallel = true;
        stmt.what
            .0
            .push(surrealdb::sql::Value::Table("workshop_items".into()));
        stmt.cond = {
            let conditions = vec![
                languages.map(|lang| {
                    sql::Expression::new(
                        sql::Value::Array(
                            vec![(lang as u8).into(), sql::Value::Number(0.into())].into(),
                        ),
                        Operator::ContainAny,
                        sql::Value::Idiom("languages".into()),
                    )
                }),
                last_updated.map(|updated| {
                    sql::Expression::new(
                        sql::Value::Idiom("last_updated".into()),
                        Operator::MoreThanOrEqual,
                        sql::Value::Number(updated.into()),
                    )
                }),
                (!tags.is_empty()).then(|| {
                    if true {
                        // All
                        sql::Expression::new(
                            sql::Value::Idiom("tags".into()),
                            Operator::ContainAll,
                            sql::Value::Array(
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
                        sql::Expression::new(
                            sql::Value::Idiom(
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
                            sql::Value::Bool(true),
                        )
                    }
                }),
                title.map(|title_query| {
                    Expression::new(
                        sql::Value::Idiom("title".into()),
                        Operator::Like,
                        sql::Value::Strand(title_query.into()),
                    )
                }),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<Expression>>();

            if conditions.is_empty() {
                None
            } else {
                let mut values = sql::Value::None;
                for mut condition in &conditions.into_iter().chunks(2) {
                    let c1 = condition.next();
                    let c2 = condition.next();
                    match (values, c1, c2) {
                        (sql::Value::None, Some(expr1), Some(expr2)) => {
                            values = sql::Value::Expression(Box::from(sql::Expression::new(
                                expr1.into(),
                                Operator::And,
                                expr2.into(),
                            )));
                        }
                        (sql::Value::None, Some(expr1), None) => {
                            values = sql::Value::Expression(Box::from(expr1));
                        }
                        (sql::Value::Expression(old), Some(expr1), Some(expr2)) => {
                            values = sql::Value::Expression(Box::from(sql::Expression::new(
                                sql::Value::Expression(old),
                                Operator::And,
                                sql::Value::Expression(Box::from(sql::Expression::new(
                                    expr1.into(),
                                    Operator::And,
                                    expr2.into(),
                                ))),
                            )));
                        }
                        (sql::Value::Expression(old), Some(expr1), None) => {
                            values = sql::Value::Expression(Box::from(sql::Expression::new(
                                sql::Value::Expression(old),
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
            use serde_json::Value;
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
