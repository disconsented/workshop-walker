use std::str::FromStr;

use salvo::{
    __private::tracing::{debug, info},
    Router,
    oapi::{Components, Operation, extract::PathParam},
    prelude::*,
};
use snafu::{OptionExt, ResultExt, Whatever};
use surrealdb::{RecordId, Surreal, engine::local::Db, rpc::Method::Select};
use tokio::sync::OnceCell;

use crate::{
    language::detect,
    model::{FullWorkshopItem, WorkshopItem, into_string},
};

static DB_POOL: OnceCell<Surreal<Db>> = OnceCell::const_new();
pub async fn start(db: Surreal<Db>) {
    DB_POOL.get_or_init(|| async { db }).await;
    let router = Router::new()
        .push(Router::with_path("api/list").get(list))
        .push(Router::with_path("api/item/{id}").get(get));
    let doc = OpenApi::new("test api", "0.0.1").merge_router(&router);
    let router = router
        .push(doc.into_router("/api-doc/openapi.json"))
        .push(SwaggerUi::new("/api-doc/openapi.json").into_router("swagger-ui"));
    let service = Service::new(router).hoop(Logger::new());

    let acceptor = TcpListener::new("127.0.0.1:5800").bind().await;
    Server::new(acceptor).serve(service).await;
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
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
        )
    }
}

#[async_trait]
impl Writer for Error {
    async fn write(mut self, _req: &mut Request, depot: &mut Depot, res: &mut Response) {
        res.render(Text::Plain(format!("Error: {:#?}", self.0)));
    }
}

#[endpoint]
async fn get(id: PathParam<String>) -> Result<Json<FullWorkshopItem>> {
    let id = id.0;
    let db: &Surreal<Db> = DB_POOL.get().unwrap();
    async fn query(id: String, db: &Surreal<Db>) -> Result<FullWorkshopItem, Whatever> {
        let id = RecordId::from_table_key("workshop_items", &id);
        let mut response = db
            .query(
                "SELECT in.appid as appid, in.description as description, in.id as id, in.title \
                 as title FROM item_dependencies WHERE out = $id",
            )
            .query(
                "SELECT out.appid as appid, out.description as description, out.id as id, \
                 out.title as title FROM item_dependencies WHERE in = $id;",
            )
            .bind(("id", id.clone()))
            .await
            .whatever_context("getting dependants")?;
        let dependants: Vec<WorkshopItem<RecordId>> =
            response.take(0).whatever_context("no dependants found")?;
        let dependencies: Vec<WorkshopItem<RecordId>> =
            response.take(1).whatever_context("no dependencies found")?;

        let result: WorkshopItem<RecordId> = db
            .select(id)
            .await
            .whatever_context("getting item")?
            .whatever_context("not found")?;

        Ok(FullWorkshopItem {
            appid: result.appid,
            description: result.description,
            id: into_string(result.id.key()),
            title: result.title,
            preview_url: result.preview_url,
            language: result.language,
            author: result.author,
            last_updated: 0,
            dependencies: dependencies
                .into_iter()
                .map(|e| WorkshopItem {
                    appid: e.appid,
                    author: e.author,
                    description: e.description,
                    id: into_string(e.id.key()),
                    language: e.language,
                    title: e.title,
                    preview_url: e.preview_url,
                    last_updated: e.last_updated,
                })
                .collect(),

            dependants: dependants
                .into_iter()
                .map(|e| WorkshopItem {
                    appid: e.appid,
                    author: e.author,
                    description: e.description,
                    id: into_string(e.id.key()),
                    language: e.language,
                    title: e.title,
                    preview_url: e.preview_url,
                    last_updated: e.last_updated,
                })
                .collect(),
        })
    }
    let results = query(id, db).await?;

    info!("Language is: {:?}", detect(&results.description));
    Ok(Json(results))
}

#[endpoint]
async fn list(req: &mut Request) -> Result<Json<Vec<WorkshopItem<String>>>> {
    let page: u64 = req.query("page").unwrap_or_default();
    let limit: u64 = req.query("limit").unwrap_or(100);
    let db: &Surreal<Db> = DB_POOL.get().unwrap();
    async fn query(
        page: u64,
        limit: u64,
        db: &Surreal<Db>,
    ) -> Result<Vec<WorkshopItem<String>>, Whatever> {
        let mut results = db
            .query("SELECT * FROM workshop_items LIMIT $limit START $start")
            .bind(("limit", limit))
            .bind(("start", limit * page))
            .await
            .whatever_context("querying")?;

        let results: Vec<WorkshopItem<RecordId>> =
            results.take(0).whatever_context("taking result")?;

        Ok(results
            .into_iter()
            .map(|res| WorkshopItem {
                appid: res.appid,
                author: res.author,
                description: res.description,
                id: res.id.key().to_string().replace("⟩", "").replace("⟨", ""),
                language: res.language,
                title: res.title,
                preview_url: res.preview_url,
                last_updated: res.last_updated,
            })
            .collect())
    }
    let results = query(page, limit, db).await?;

    Ok(Json(results))
}
