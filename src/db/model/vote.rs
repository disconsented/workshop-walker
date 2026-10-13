use chrono::{DateTime, Utc};
use macros::dual_struct;
use serde::{Deserialize, Serialize};
use surrealdb_types::SurrealValue;

use crate::db::{AppID, IAppID};
/// A voting record
#[dual_struct(derive(Serialize, Deserialize, Clone, Debug))]
pub struct Vote {
    /// The app this is associated with, for possible filtering
    #[dual_type(IAppID)]
    pub app: AppID,
    pub score: f32,
    pub when: DateTime<Utc>,
}
