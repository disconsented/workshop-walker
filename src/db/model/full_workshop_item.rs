use serde::{Deserialize, Serialize};
use surrealdb_types::SurrealValue;
use tracing::error;
use twinned::dual_struct;

use crate::{
    db::{
        AppID, IAppID, IItemID, ItemID,
        model::{
            ExternalTag, ExternalWorkshopItemProperties, InternalTag,
            InternalWorkshopItemProperties, to_external_props, to_external_tag, to_internal_props,
            to_internal_tag,
            username::{
                ExternalUsername, InternalUsername, to_external_username, to_internal_username,
            },
        },
    },
    processing::language_actor::DetectedLanguage,
};

// Read-only, dual still needed for ID conversion
#[dual_struct(derive(Serialize, Deserialize, Clone, Debug))]
pub struct FullWorkshopItem {
    // Core identifiers
    #[dual_type(IItemID)]
    pub id: ItemID, // The item's ID
    #[dual_type(IAppID)]
    pub app: AppID, // The steam ID of the app this belongs to

    // Content information
    pub title: String,       // The titles name
    pub description: String, // HTML encoded description from steam
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview_url: Option<String>, // The URL to the banner image

    // Metadata and categorization
    #[serde(default)]
    #[dual_type(Vec<InternalTag>, to_external = to_external_tag, to_internal = to_internal_tag)]
    pub tags: Vec<ExternalTag>, // The list of tags found
    #[dual_type(Vec<InternalWorkshopItemProperties>, to_external = to_external_props, to_internal = to_internal_props)]
    #[serde(default)]
    pub properties: Vec<ExternalWorkshopItemProperties>, // Approved or owned properties
    pub score: f32, // The "quality" score assigned by steam

    // Author and timing
    #[dual_type(Option<InternalUsername>, to_external = to_external_username, to_internal = to_internal_username)]
    pub author: Option<ExternalUsername>, // Authors steam ID
    pub last_updated: u64, // Timestamp in milliseconds
    pub created: u64,      // Timestamp in milliseconds
    pub ml_last_run: Option<u64>,

    // Localization
    #[serde(default)]
    pub languages: Vec<DetectedLanguage>, // All languages found in the items description

    // Dependencies
    #[dual_type(Vec<InternalFullWorkshopItem>, to_external = to_external_full_item, to_internal = to_internal_full_item, surreal(wrap))]
    #[serde(default)]
    pub dependencies: Vec<ExternalFullWorkshopItem>, // A list of dependencies found
    #[dual_type(Vec<InternalFullWorkshopItem>, to_external = to_external_full_item, to_internal = to_internal_full_item, surreal(wrap))]
    #[serde(default)]
    pub dependants: Vec<ExternalFullWorkshopItem>, // A list of dependants found

    // Raw metrics
    pub lifetime_subscriptions: u64,
    pub subscriptions: u64,
    pub views: u64,
    // Historical metrics
    pub view_history: Vec<u64>,
    pub subscription_history: Vec<u64>,
    // Calculations
    // subscriptions/lifetime_subscriptions
    pub retention: f32,
    // Subscriptions/Views
    pub conversions: f32,

    // Calculated
    pub hotness: f32,
    pub trend_week: f32,
    pub trend_month: f32,
    pub trend_quarter: f32,
    pub trend_half: f32,
    pub trend_year: f32,
}

fn to_external_full_item(
    internal: Vec<InternalFullWorkshopItem>,
) -> Result<Vec<ExternalFullWorkshopItem>, surrealdb_types::Error> {
    internal
        .into_iter()
        .map(TryFrom::try_from)
        .collect::<Result<_, _>>()
        .inspect_err(|error| error!(?error, "to_external_full_item"))
}

fn to_internal_full_item(external: Vec<ExternalFullWorkshopItem>) -> Vec<InternalFullWorkshopItem> {
    external.into_iter().map(Into::into).collect()
}
