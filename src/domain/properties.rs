use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};
use snafu::prelude::*;

use crate::db::model::{Class, Source};

#[derive(Debug, Snafu, Clone)]
#[non_exhaustive]
pub enum PropertiesError {
    #[snafu(display("Invalid vote score"))]
    InvalidVoteScore,
    #[snafu(display("Bad request: {msg}"))]
    BadRequest { msg: String },
    #[snafu(display("Conflict"))]
    Conflict,
    #[snafu(display("Internal error"))]
    Internal,
}

/// Data required to create/link a new property to a workshop item
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NewProperty {
    pub workshop_item: String,
    pub class: Class,
    pub value: String,
    /// Reasoning or justification for an inclusion
    pub note: Option<String>,
}

/// Data required to cast or update a vote on a property
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VoteData {
    pub item: String,
    pub class: Class,
    pub value: String,
    pub score: i32,
}

/// Port for property-related persistence operations.
pub trait PropertiesPort: Send + Sync + 'static {
    async fn create_or_link_property(
        &self,
        new_prop: NewProperty,
        source: Source<String>,
    ) -> Result<(), PropertiesError>;
    async fn vote(&self, vote: VoteData, userid: String) -> Result<(), PropertiesError>;
    async fn remove_vote(&self, vote: VoteData, userid: String) -> Result<(), PropertiesError>;
}
