use macros::dual_struct;
use serde::{Deserialize, Serialize};
use surrealdb_types::SurrealValue;
use tracing::error;

use crate::db::{ITagID, TagID};
#[dual_struct(derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Hash,
    Ord,
    PartialOrd
))]
pub struct Tag {
    #[dual_type(ITagID)]
    pub id: TagID,
    pub display_name: String,
    // Used to filter tags that aren't used any more or the weird one offs that show up
    pub known_members: i64,
}

pub(crate) fn to_external_tag(
    internal: Vec<InternalTag>,
) -> Result<Vec<ExternalTag>, surrealdb_types::Error> {
    internal
        .into_iter()
        .map(ExternalTag::try_from)
        .collect::<Result<_, _>>()
        .inspect_err(|error| error!(?error, "to_external_tag"))
}

pub(crate) fn to_internal_tag(external: Vec<ExternalTag>) -> Vec<InternalTag> {
    external.into_iter().map(InternalTag::from).collect()
}
