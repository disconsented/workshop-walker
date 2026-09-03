use salvo::oapi::ToSchema;
use serde::{Deserialize, Serialize};
use snafu::prelude::*;

use crate::db::{
    ItemID, UserID,
    model::{InternalUser, InternalWorkshopItemProperties, Property, Status},
};

#[derive(Debug, Snafu, Clone)]
#[non_exhaustive]
pub enum AdminError {
    #[snafu(display("Bad request: {msg}"))]
    BadRequest { msg: String },
    #[snafu(display("Conflict"))]
    Conflict,
    #[snafu(display("Internal error"))]
    Internal,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct PatchUserData {
    pub id: UserID,
    pub banned: Option<bool>,
    pub admin: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct PatchRelationshipData {
    pub item: ItemID,
    #[serde(flatten)]
    pub property: Property,
    pub status: Status,
}

/// Port for admin-related persistence operations.
pub trait AdminPort: Send + Sync + 'static {
    async fn list_users(&self) -> Result<Vec<InternalUser>, AdminError>;
    async fn patch_user(&self, patch: PatchUserData) -> Result<(), AdminError>;
    async fn list_workshop_item_properties(
        &self,
    ) -> Result<Vec<InternalWorkshopItemProperties>, AdminError>;
    async fn patch_workshop_item_property(
        &self,
        patch: PatchRelationshipData,
    ) -> Result<(), AdminError>;
}
