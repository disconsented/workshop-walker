use lingua::Language;
use salvo::{
    oapi::{Components, RefOr, Schema},
    prelude::ToSchema,
};
use serde::{Deserialize, Serialize};
use surrealdb::{RecordId, RecordIdKey};

use crate::language::DetectedLanguage;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tag {
    app_id: u64,
    display_name: String,
    tag: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct WorkshopItem<ID> {
    pub appid: i64,
    pub author: String,
    pub description: String,
    pub id: ID,
    pub language: DetectedLanguage,
    pub last_updated: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_url: Option<String>,
    pub title: String,
}
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct FullWorkshopItem {
    pub appid: i64,
    pub author: String,
    pub dependants: Vec<WorkshopItem<String>>,
    pub dependencies: Vec<WorkshopItem<String>>,
    pub description: String,
    pub id: String,
    pub language: DetectedLanguage,
    pub last_updated: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_url: Option<String>,
    pub title: String,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Dependencies {
    #[serde(rename = "in")]
    pub this: RecordId,
    #[serde(rename = "out")]
    pub dependency: RecordId,
}

pub fn into_string(key: &RecordIdKey) -> String{
    key.to_string().replace("⟩", "").replace("⟨", "")
}