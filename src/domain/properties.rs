use macros::dual_struct;
use serde::{Deserialize, Serialize};
use snafu::prelude::*;
use surrealdb_types::SurrealValue;

use crate::db::{
    IItemID, IUserID, ItemID,
    model::{Class, InternalSource, InternalUser, Status},
};

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
#[dual_struct(derive(Serialize, Deserialize, Clone, Debug))]
pub struct NewProperty {
    #[dual_type(IItemID)]
    pub workshop_item: ItemID,
    pub class: Class,
    pub value: String,
    /// Reasoning or justification for an inclusion
    pub note: Option<String>,
}

/// Data required to cast or update a vote on a property
#[dual_struct(derive(Serialize, Deserialize, Clone, Debug))]
pub struct VoteData {
    #[dual_type(IItemID)]
    pub item: ItemID,
    pub class: Class,
    pub value: String,
    pub score: i32,
}

/// Port for property-related persistence operations.
pub trait PropertiesPort: Send + Sync + 'static {
    async fn create_or_link_property(
        &self,
        new_prop: InternalNewProperty,
        source: InternalSource,
        status: Status,
    ) -> Result<(), PropertiesError>;
    async fn vote(&self, vote: InternalVoteData, userid: IUserID) -> Result<(), PropertiesError>;
    async fn remove_vote(
        &self,
        vote: InternalVoteData,
        userid: IUserID,
    ) -> Result<(), PropertiesError>;
}
