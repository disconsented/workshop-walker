use surrealdb::{IndexedResults, Surreal, engine::local::Db};
use tracing::{debug, error};

use crate::{
    db::{IAppID, model::InternalApp},
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
    async fn list_available(&self) -> Result<Vec<InternalApp>, AppError> {
        match self
            .db
            .query("SELECT *, tags.*, default_tags.* FROM apps WHERE available = true;")
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

    async fn upsert(&self, app: InternalApp) -> Result<(), AppError> {
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
        if let Err(error) = self.db.query("DELETE $id").bind(("id", id)).await {
            error!(?error, "failed to remove app");
            return Err(AppError::Internal);
        }
        Ok(())
    }

    async fn list(&self) -> Result<Vec<InternalApp>, AppError> {
        match self
            .db
            .query("SELECT *, tags.*, default_tags.* FROM apps")
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

    async fn get(&self, id: IAppID) -> Result<InternalApp, AppError> {
        match self
            .db
            .query("SELECT *, tags.*, default_tags.* FROM $id")
            .bind(("id", id))
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
