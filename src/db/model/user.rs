use chrono::{DateTime, Utc};
use macros::dual_struct;
use serde::{Deserialize, Serialize, Serializer};
use surrealdb_types::SurrealValue;

use crate::db::{
    IUserID, UserID,
    model::username::{
        ExternalUsername, InternalUsername, to_external_username, to_internal_username,
    },
};
/// A workshop walker user
#[dual_struct(derive(Serialize, Deserialize, Clone, Debug))]
pub struct User {
    /// The steam account ID
    #[dual_type(IUserID)]
    pub id: UserID,
    #[dual_type(Option<InternalUsername>, to_external = to_external_username, to_internal = to_internal_username)]
    pub username: Option<ExternalUsername>,
    /// Privileged access
    pub admin: bool,
    pub banned: bool,
    /// UTC timestamp of when the user last logged in
    // Surrealdb bug: https://github.com/surrealdb/surrealdb/issues/3550
    #[serde(serialize_with = "serialize_chrono_as_sql_datetime")]
    pub last_logged_in: DateTime<Utc>,
}

pub fn serialize_chrono_as_sql_datetime<S>(x: &DateTime<Utc>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    Into::<surrealdb_types::Datetime>::into(*x).serialize(s)
}
