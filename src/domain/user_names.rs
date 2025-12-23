use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use snafu::prelude::*;
use surrealdb::RecordId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserName {
    pub id: RecordId,
    #[serde(serialize_with = "crate::db::model::serialize_chrono_as_sql_datetime")]
    pub last_updated: DateTime<Utc>,
    pub name: String,
}

#[derive(Debug, Snafu, Clone)]
#[non_exhaustive]
pub enum UserNameError {
    #[snafu(display("Internal error: {msg}"))]
    Internal { msg: String },
    #[snafu(display("Not found"))]
    NotFound,
}

pub trait UserNamesPort: Send + Sync + 'static {
    async fn upsert(&self, username: UserName) -> Result<(), UserNameError>;
    async fn get_by_id(&self, id: RecordId) -> Result<Option<UserName>, UserNameError>;
}
