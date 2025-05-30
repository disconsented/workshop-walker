use std::fmt::{Display, Formatter};

use salvo::prelude::ToSchema;
use serde::{Deserialize, Serialize};
use surrealdb::{RecordId, RecordIdKey};

use crate::language::DetectedLanguage;
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, Default)]
pub enum OrderBy {
    Alphabetical,
    #[default]
    LastUpdated,
    Score,
    Dependents,
}

impl OrderBy {
    pub fn column_name(&self) -> &str {
        match self {
            OrderBy::Alphabetical => "title",
            OrderBy::LastUpdated => "last_updated",
            OrderBy::Score => "score",
            OrderBy::Dependents => "dependencies_length",
        }
    }
}

impl Display for OrderBy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct Tag {
    pub app_id: u64,
    pub display_name: String,
    #[serde(rename = "id")]
    pub tag: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct WorkshopItem<ID> {
    pub appid: i64,
    pub author: String,
    pub description: String,
    pub id: ID,
    pub languages: Vec<DetectedLanguage>,
    pub last_updated: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_url: Option<String>,
    pub title: String,
    pub tags: Vec<Tag>,
    pub score: f32,
}
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct FullWorkshopItem {
    pub appid: i64, // The steam ID of the app this belongs to
    pub author: String, //Authors steam ID
    pub dependants: Vec<FullWorkshopItem>, // A list of dependants found
    pub dependencies: Vec<FullWorkshopItem>, // A list of dependencies found
    pub description: String, // HTML encoded description from steam
    pub id: String, // The item's ID
    pub languages: Vec<DetectedLanguage>, // All languages found in the items description
    pub last_updated: u64, // Timestamp in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_url: Option<String>, // The URL to the banner image
    pub title: String, // The titles name
    pub tags: Vec<Tag>, // The list of tags found
    pub score: f32, // The "quality" score assigned by steam
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Dependencies {
    pub id: RecordId,
    #[serde(rename = "in")]
    pub this: RecordId,
    #[serde(rename = "out")]
    pub dependency: RecordId,
}

pub fn into_string(key: &RecordIdKey) -> String {
    key.to_string().replace("⟩", "").replace("⟨", "")
}
