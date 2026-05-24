use std::{env, path::Path, sync::Arc};

use snafu::{prelude::*, Whatever};
use surrealdb::{
    engine::{
        any::{connect, Any},
        local::{Db, RocksDb},
    }, opt::{auth::Root, IntoEndpoint},
    Connection,
    Surreal,
};
use tokio_stream::StreamExt;
use surrealkit::EmbeddedSchemaFile;
use tokio_stream::wrappers::ReadDirStream;
use tracing::{debug, error, info_span, Instrument};
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

    // debug!("checking migrations");
    //
    // debug!("migrations finished");

    let db = setup_database(&settings).await?;
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
    // Another frustrating limitation with this ecosystem, surrealkit SPECIFICALLY
    // needs Surreal<Any> which cannot be casted or converted into, which wouldn't
    // be a massive issue if connect also didnt return a concerte type :|
    // So, now, we double open or I throw together my own migration code
    {
        debug!("running migrations");
        let db = surrealdb::engine::any::connect("./workshopdb")
            .await
            .whatever_context("connecting to 'any' db")?;
        db.use_ns("workshop")
            .use_db("workshop")
            .await
            .whatever_context("using ns/db")?;

        let files = tokio::fs::read_dir("./migrations")
            .await
            .whatever_context("reading migrations directory")?;
        let files = ReadDirStream::new(files)
            .filter_map(|dir| dir.ok().map(|dir| EmbeddedSchemaFile {
                path: dir.path(),
                sql: "",
            })
            .collect::<Vec<_>>();

        surrealkit::run_sync();

        surrealkit::run_sync_embedded(&db, &files)
            .await
            .whatever_context("syncing surreal")?;
        debug!("migrations finished");
    }

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
