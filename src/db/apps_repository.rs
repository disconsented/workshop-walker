use surrealdb::{Surreal, engine::local::Db, IndexedResults};
use tracing::{debug, error};

use crate::{
    db::{
        AppID, IAppID,
        model::{App, TagRef},
    },
    domain::apps::{AppError, AppsPort},
};

pub struct AppsSilo {
    pub db: Surreal<Db>,
}

impl AppsSilo {
    pub fn new(db: Surreal<Db>) -> Self {
        Self { db }
    }
}

impl AppsPort for AppsSilo {
    async fn list_available(&self) -> Result<Vec<App<TagRef>>, AppError> {
        match self
            .db
            .query(
                "SELECT *, tags.map(|$v|{id: $v.to_string(), display_name: $v.id().to_string()}), \
                 default_tags.map(|$v|{id: $v.to_string(), display_name: $v.id().to_string()}), \
                 id.id() FROM apps WHERE available = true",
            )
            .await
            .map(|mut q| q.take(0))
        {
            Ok(Ok(results)) => Ok(results),
            Ok(Err(error)) | Err(error) => {
                error!(?error, "failed to list available apps");
                Err(AppError::Internal)
            }
        }
    }

    async fn upsert(&self, app: App<TagRef>) -> Result<(), AppError> {
        match self
            .db
            .query("UPSERT apps CONTENT $app")
            .bind(("app", app.clone()))
            .await
            .map(IndexedResults::check)
        {
            Ok(Ok(response)) => {
                debug!(?app, ?response, "upserted app");
                Ok(())
            }
            Err(error) | Ok(Err(error)) => {
                error!(?error, "failed to upsert app");
                Err(AppError::Internal)
            }
        }
    }

    async fn remove(&self, id: IAppID) -> Result<(), AppError> {
        if let Err(error) = self
            .db
            .query("DELETE $id")
            .bind(("id", id))
            .await
        {
            error!(?error, "failed to remove app");
            return Err(AppError::Internal);
        }
        Ok(())
    }

    async fn list(&self) -> Result<Vec<App<TagRef>>, AppError> {
        match self
            .db
            .query(
                "SELECT *, tags.map(|$v|{id: $v.to_string(), display_name: $v.id().to_string()}), \
                 default_tags.map(|$v|{id: $v.to_string(), display_name: $v.id().to_string()}), \
                 id.id() FROM apps",
            )
            .await
            .map(|mut q| q.take(0))
        {
            Ok(Ok(results)) => Ok(results),
            Ok(Err(e)) | Err(e) => {
                error!(?e, "failed to list apps");
                Err(AppError::Internal)
            }
        }
    }

    async fn get(&self, id: u32) -> Result<App<TagRef>, AppError> {
        match self
            .db
            .query(
                "SELECT *, tags.map(|$v|{id: $v.to_string(), display_name: $v.id().to_string()}), \
                 default_tags.map(|$v|{id: $v.to_string(), display_name: $v.id().to_string()}), \
                 id.id() FROM $id",
            )
            .bind(("id", IAppID::from(id as u64)))
            .await
            .map(|mut q| q.take(0))
        {
            Ok(Ok(Some(app))) => Ok(app),
            Ok(Ok(None)) => Err(AppError::NotFound),
            Ok(Err(error)) | Err(error) => {
                error!(?error, "failed to get app");
                Err(AppError::Internal)
            }
        }
    }
}
