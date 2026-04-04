use snafu::prelude::*;
use crate::db::IAppID;
use crate::db::model::{App, TagRef};

#[derive(Debug, Snafu, Clone)]
#[non_exhaustive]
pub enum AppError {
    #[snafu(display("Bad request: {msg}"))]
    BadRequest { msg: String },
    #[snafu(display("Conflict"))]
    Conflict,
    #[snafu(display("Internal error"))]
    Internal,
    #[snafu(display("Not found"))]
    NotFound,
}

/// Port for app-related persistence operations.
pub trait AppsPort: Send + Sync + 'static {
    async fn list_available(&self) -> Result<Vec<App<TagRef>>, AppError>;
    async fn upsert(&self, app: App<TagRef>) -> Result<(), AppError>;
    async fn remove(&self, id: IAppID) -> Result<(), AppError>;
    async fn list(&self) -> Result<Vec<App<TagRef>>, AppError>;
    async fn get(&self, id: IAppID) -> Result<App<TagRef>, AppError>;
}
