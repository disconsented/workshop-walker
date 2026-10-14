use serde::{Deserialize, Serialize};
use surrealdb_types::SurrealValue;
use twinned::dual_struct;

use crate::db::{
    IItemID, ItemID,
    model::{InternalSource, property::Property, source::ExternalSource, status::Status},
};
#[dual_struct(derive(Serialize, Deserialize, Clone, Debug))]
pub struct WorkshopItemProperties {
    #[dual_type(IItemID, surreal(wrap))]
    #[serde(rename = "in")]
    pub r#in: ItemID,
    #[serde(rename = "out")]
    pub out: Property,
    /// Reasoning or justification for an inclusion
    pub note: Option<String>,
    pub status: Status,
    /// The current score
    pub upvote_count: i64,
    /// The total upvotes
    pub vote_count: u64,
    #[dual_type(InternalSource)]
    pub source: ExternalSource,
    pub vote_state: Option<i32>,
}

pub(crate) fn to_external_props(
    internal: Vec<InternalWorkshopItemProperties>,
) -> Result<Vec<ExternalWorkshopItemProperties>, surrealdb_types::Error> {
    internal
        .into_iter()
        .map(TryFrom::try_from)
        .collect::<Result<_, _>>()
}

pub(crate) fn to_internal_props(
    external: Vec<ExternalWorkshopItemProperties>,
) -> Vec<InternalWorkshopItemProperties> {
    external.into_iter().map(From::from).collect()
}
