mod admin;
pub mod auth;
mod companions;
pub mod item;
pub mod properties;
mod query;

use std::sync::Arc;

use salvo::{
    oapi::{Components, Operation},
    prelude::*,
    Router,
};
use snafu::Whatever;
use surrealdb::{engine::local::Db, Surreal};
use tokio::sync::OnceCell;

use crate::app_config::Config;

/// Global
static DB_POOL: OnceCell<Surreal<Db>> = OnceCell::const_new();
///  Start the webserver returning once it exists
pub async fn start(config: Arc<Config>) {
    let router = Router::new().push(
        Router::with_path("api")
            .hoop(max_size(1024 * 1024))
            .push(Router::with_path("list").get(query::list))
            .push(
                Router::with_path("item/{id}")
                    .hoop(auth::validate_opt)
                    .get(item::get),
            )
            .push(
                Router::with_path("property")
                    .hoop(auth::validate_biscuit_token)
                    .post(properties::new),
            )
            .push(
                Router::with_path("vote")
                    .hoop(auth::validate_biscuit_token)
                    .push(
                        Router::with_path("property")
                            .post(properties::vote)
                            .delete(properties::remove),
                    ),
            )
            .push(
                Router::with_path("admin")
                    .hoop(auth::validate_biscuit_token)
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
            .push(Router::with_path("login").get(auth::redirect_to_steam_auth))
            .push(Router::with_path("verify").get(auth::verify_token_from_steam))
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
    async fn write(mut self, _: &mut Request, _: &mut Depot, res: &mut Response) {
        res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
        res.render(Text::Plain(format!("Error: {:#?}", self.0)));
    }
}
