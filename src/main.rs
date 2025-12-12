use std::{env, sync::Arc};

use salvo::__private::tracing::debug;
use snafu::{Whatever, prelude::*};
use surrealdb::{Surreal, engine::local::RocksDb, opt::auth::Root};
use surrealdb_migrations::MigrationRunner;
use tracing::{Instrument, error, info_span};
use tracing_subscriber::fmt::format::FmtSpan;

use crate::{application::admin_service::AdminService, db::admin_repository::AdminSilo};

mod actors;
mod app_config;
mod application;
mod apps;
mod db;
mod domain;
mod processing;
mod steam;
mod web;

pub type Result<T, E = Error> = std::result::Result<T, E>;
pub type Error = Whatever;
#[tokio::main]
async fn main() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(env::var("RUST_LOG").unwrap_or_default())
        .with_span_events(FmtSpan::CLOSE)
        .try_init();
    let settings: app_config::Config = config::Config::builder()
        .add_source(config::File::with_name("config/config.toml"))
        .build()
        .whatever_context("finding config")?
        .try_deserialize()
        .whatever_context("deserializing config")?;
    let span = info_span!("spawn");
    let db = Surreal::new::<RocksDb>("./workshopdb")
        .await
        .whatever_context("connecting to db")?;

    // Select a specific namespace / database
    db.use_ns("workshop")
        .use_db("workshop")
        .await
        .whatever_context("using ns/db")?;
    db.query(format!(
        "DEFINE USER IF NOT EXISTS {} ON ROOT PASSWORD '{}' ROLES OWNER DURATION FOR TOKEN 1h, \
         FOR SESSION NONE;",
        settings.database.user, settings.database.password
    ))
    .await
    .whatever_context("creating root user")?;

    // Signin as db user (root)
    db.signin(Root {
        username: &settings.database.user,
        password: &settings.database.password,
    })
    .await
    .whatever_context("signing in to db")?;

    debug!("checking migrations");
    // Run migrations
    MigrationRunner::new(&db)
        .up()
        .instrument(info_span!(parent: &span, "run migrations"))
        .await
        .whatever_context("Failed to apply migrations")?;
    debug!("migrations finished");
    {
        let admin_service = AdminService::new(AdminSilo::new(db.clone()));
        for user in &settings.admin_users {
            debug!(%user, "Setting admin flag for user");
            let _ = admin_service
                .patch_user(domain::admin::PatchUserData {
                    id: user.clone(),
                    banned: None,
                    admin: Some(true),
                })
                .await
                .inspect_err(|error| error!(?error, %user, "Failed to set admin flag for user"));
        }
    }

    actors::spawn(&settings, &db)
        .instrument(info_span!(parent: &span, "spawn actors"))
        .await?;
    web::start(db, Arc::new(settings)).await;
    Ok(())
}

/// Inserts data from either the disk cache (for development) or from stream
/// directly. Also converts BB code into markdown.

#[cfg(test)]
mod test {
    use serde::Serialize;

    #[test]
    fn test_serialize_ordering() {
        #[derive(Serialize)]
        pub enum Ordering {
            Order(Vec<bool>),
            Random,
        }

        dbg!(serde_json::to_string(&Ordering::Order(vec![true])).unwrap());
    }
}
