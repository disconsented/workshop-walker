use macros::dual_struct;
use serde::{Deserialize, Serialize};
use surrealdb_types::SurrealValue;

use crate::{
    db::{
        AppID, IAppID, IItemID, ItemID,
        model::{
            ExternalTag, ExternalUsername, ExternalWorkshopItemProperties, InternalTag,
            InternalUsername, InternalWorkshopItemProperties, to_external_props, to_external_tag,
            to_internal_props, to_internal_tag,
        },
    },
    processing::language_actor::DetectedLanguage,
};

#[dual_struct(derive(Serialize, Deserialize, Clone, Debug))]
pub struct WorkshopItem {
    #[dual_type(IAppID)]
    pub app: AppID,
    #[dual_type(InternalUsername)]
    pub author: ExternalUsername,
    pub description: String,
    #[dual_type(IItemID)]
    pub id: ItemID,
    pub languages: Vec<DetectedLanguage>,
    pub last_updated: u64,
    pub created: u64,
    pub ml_last_run: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_url: Option<String>,
    pub title: String,
    #[dual_type(Vec<InternalTag>, to_external = to_external_tag, to_internal = to_internal_tag)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<ExternalTag>,
    pub score: f32,
    #[dual_type(Vec<InternalWorkshopItemProperties>, to_external = to_external_props, to_internal = to_internal_props, surreal(wrap))]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub properties: Vec<ExternalWorkshopItemProperties>,
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
