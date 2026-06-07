extern crate core;
extern crate alloc;

use std::{env, sync::Arc};

use migrations_tool::{Migrator, Outcome};
use snafu::{Whatever, prelude::*};
use surrealdb::{
    Surreal,
    engine::local::{Db, RocksDb},
    opt::auth::Root,
};
use tokio_stream::StreamExt;
use tracing::{Instrument, debug, error, info_span};
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

    let db = setup_database(&settings)
        .await
        .inspect_err(|error| error!(?error, "Failed to setup db"))?;
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

async fn setup_database(settings: &app_config::Config) -> Result<Surreal<Db>, Error> {
    let db = Surreal::new::<RocksDb>("./workshopdb".to_string())
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
        username: settings.database.user.clone(),
        password: settings.database.password.clone(),
    })
    .await
    .whatever_context("signing in to db")?;

    let plan = Migrator::from_files("./migrations")
        .whatever_context("loading migrations")?
        .with_table("_migrations")
        .ignore_checksum_changes(false)
        .validate()
        .whatever_context("validating migrations")?
        .plan(&db)
        .await
        .whatever_context("planning migrations")?;

    debug!("will apply {} migrations", plan.pending().len());

    {
        // The stream is not Unpin; pin it before polling.
        let mut stream = std::pin::pin!(plan.execute(&db));
        while let Some(outcome) = stream.next().await {
            match outcome.whatever_context("applying migration")? {
                Outcome::Applied { id, duration } => {
                    debug!("applied {id} in {:?}", duration);
                }
                Outcome::Skipped { id, .. } => {
                    debug!("skipped {id}");
                }
            }
        }
    }

    Ok(db)
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
        }

        dbg!(serde_json::to_string(&Ordering::Order(vec![true])).unwrap());
    }
}
