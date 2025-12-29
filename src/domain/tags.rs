use snafu::prelude::*;

use crate::db::model::Tag;

#[derive(Debug, Snafu, Clone)]
#[non_exhaustive]
pub enum TagError {
    #[snafu(display("Internal error: {msg}"))]
    Internal { msg: String },
    #[snafu(display("Not found"))]
    NotFound,
}

pub trait TagsPort: Send + Sync + 'static {
    async fn upsert_tags(&self, app_id: u32, tags: Vec<Tag>) -> Result<(), TagError>;
}
