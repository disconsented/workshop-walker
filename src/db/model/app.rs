use serde::{Deserialize, Serialize};
use surrealdb_types::SurrealValue;
use tracing::error;
use twinned::dual_struct;

use crate::db::{AppID, IAppID, ITagID, TagID};
/// A steam workshop app
#[dual_struct(derive(Serialize, Deserialize, Clone, Debug))]
pub struct App {
    /// The steam ID for an app
    #[dual_type(IAppID)]
    pub id: AppID,
    /// App name, I.E. Rimworld
    pub name: String,
    /// The developers primary name I.E. Ludeon Studios
    pub developer: String,
    pub description: String,
    /// Banner image URL
    pub banner: String,
    /// Can the app be interacted with for facets, votes & companions
    pub enabled: bool,
    /// Whether the app is visible on the index
    pub available: bool,
    /// List of tags to select by default
    #[serde(default)]
    #[dual_type(Vec<ITagID>, to_external = to_external_tag_id, to_internal = to_internal_tag_id)]
    pub default_tags: Vec<TagID>,
    /// List of known tags
    #[serde(default)]
    #[dual_type(Vec<ITagID>, to_external = to_external_tag_id, to_internal = to_internal_tag_id)]
    pub tags: Vec<TagID>,
}

fn to_external_tag_id(internal: Vec<ITagID>) -> Result<Vec<TagID>, surrealdb_types::Error> {
    internal
        .into_iter()
        .map(TagID::try_from)
        .collect::<Result<_, _>>()
        .inspect_err(|error| error!(?error, "to_external_tag"))
}

fn to_internal_tag_id(external: Vec<TagID>) -> Vec<ITagID> {
    external.into_iter().map(ITagID::from).collect()
}
