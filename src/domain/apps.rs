use snafu::prelude::*;

use crate::db::model::App;

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
    async fn list_available(&self) -> Result<Vec<App>, AppError>;
    async fn upsert(&self, app: App) -> Result<(), AppError>;
    async fn remove(&self, id: u32) -> Result<(), AppError>;
    async fn list(&self) -> Result<Vec<App>, AppError>;
    async fn get(&self, id: u32) -> Result<App, AppError>;
}
