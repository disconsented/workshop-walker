use std::fmt::{Display, Formatter};

use chrono::{DateTime, Utc};
use salvo::prelude::ToSchema;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
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
    pub appid: i64,                          // The steam ID of the app this belongs to
    pub author: String,                      // Authors steam ID
    pub dependants: Vec<FullWorkshopItem>,   // A list of dependants found
    pub dependencies: Vec<FullWorkshopItem>, // A list of dependencies found
    pub description: String,                 // HTML encoded description from steam
    pub id: String,                          // The item's ID
    pub languages: Vec<DetectedLanguage>,    // All languages found in the items description
    pub last_updated: u64,                   // Timestamp in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_url: Option<String>, // The URL to the banner image
    pub title: String,                       // The titles name
    pub tags: Vec<Tag>,                      // The list of tags found
    pub score: f32,                          // The "quality" score assigned by steam
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

/// A steam workshop app
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct App {
    /// The steam ID for an app
    pub id: u32,
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
    pub default_tags: Vec<()>,
}

/// A workshop walker user
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    /// The steam account ID
    pub id: u64,
    /// Privileged access
    pub admin: bool,
    pub banned: bool,
    /// UTC timestamp of when the user last logged in
    pub last_logged_in: DateTime<Utc>,
}

/// Crowdsourced metadata for an item, private version
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Property {
    /// Snowflake generated ID
    pub id: String,
    /// Associated app ID, for enforcing uniqueness
    pub app_id: u32,
    pub class: Class,
    pub value: String,
    /// Reasoning or justification for an inclusion
    pub note: Option<String>,
    pub status: Status,
    pub upvote_count: u64,
    pub vote_count: u64,
    /// The item that this is associated with
    pub workshop_item: RecordId,
}

/// Crowdsourced relationships for an item, used for "soft" dependencies not
/// supplied by steam, private version
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Companion {
    /// Snowflake generated ID
    pub id: String,
    pub r#in: RecordId,
    pub out: RecordId,
    /// Reasoning or justification for an inclusion
    pub note: Option<String>,
    pub status: Status,
    pub upvote_count: u64,
    pub vote_count: u64,
    /// The item that this is associated with
    pub workshop_item: RecordId,
}

/// A voting record
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Vote {
    /// The app this is associated with, for possible filtering
    pub app_id: String,
    pub link: RecordId,
    pub score: f32,
    pub user: RecordId,
    pub when: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum Source {
    /// Auto-generated
    System,
    /// User submitted
    User(RecordId),
}

#[derive(Debug, ToSchema, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub enum Class {
    /// Anything like addon, overhaul, bugfix, patch
    r#Type,
    /// Literary themes like mecha
    Theme,
    /// Literary genres like `CyberPunk`
    Genre,
    /// Mod features, like "new scenario" or "new clothes"
    Feature,
}

#[derive(
    Debug,
    Default,
    ToSchema,
    Copy,
    Clone,
    Serialize_repr,
    Deserialize_repr,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
)]
#[repr(i8)]
pub enum Status {
    Rejected = -1,
    #[default]
    Pending = 0,
    Accepted = 1,
}
