use snafu::prelude::*;

use crate::db::{model::InternalTag, IAppID, ITagID};

#[derive(Debug, Snafu, Clone)]
#[non_exhaustive]
pub enum TagError {
    #[snafu(display("Internal error: {msg}"))]
    Internal { msg: String },
    #[snafu(display("Not found"))]
    NotFound,
}

pub trait TagsPort: Send + Sync + 'static {
    async fn upsert_tags(&self, app: IAppID, tags: Vec<InternalTag>) -> Result<(), TagError>;

    async fn set_tag_known_members(&self, tag: ITagID, members: i64) -> Result<(), TagError>;
}
