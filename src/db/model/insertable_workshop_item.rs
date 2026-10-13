use serde::{Deserialize, Serialize};
use surrealdb_core::types::SurrealValue;

use crate::{
    db::{IAppID, IItemID, ITagID, IUsernameID},
    processing::language_actor::DetectedLanguage,
};

// Gave up trying to work around errors with missing fields (tags & properties)
// on insert
#[derive(Serialize, Deserialize, Clone, Debug, SurrealValue)]
pub struct InsertableWorkshopItem {
    pub app: IAppID,
    pub author: IUsernameID,
    pub description: String,
    pub id: IItemID,
    pub languages: Vec<DetectedLanguage>,
    pub last_updated: u64,
    pub created: u64,
    pub preview_url: Option<String>,
    pub title: String,
    pub score: f32,
    pub tags: Vec<ITagID>,
    pub hidden: bool,

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
