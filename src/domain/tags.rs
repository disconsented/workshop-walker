use snafu::prelude::*;
use crate::db::IAppID;
use crate::db::model::{InternalTag};

#[derive(Debug, Snafu, Clone)]
#[non_exhaustive]
pub enum TagError {
    #[snafu(display("Internal error: {msg}"))]
    Internal { msg: String },
    #[snafu(display("Not found"))]
    NotFound,
}

pub trait TagsPort: Send + Sync + 'static {
    async fn upsert_tags(&self, app_id: IAppID, tags: Vec<InternalTag>) -> Result<(), TagError>;
}
