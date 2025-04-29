use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Struct2 {
    pub publishedfileid: String,
    pub sortorder: i64,
    pub file_type: i64,
}

#[derive(Serialize, Deserialize)]
struct Struct1 {
    pub tag: String,
    pub display_name: String,
}

#[allow(clippy::missing_docs_in_private_items)]
#[derive(Serialize, Deserialize)]
struct Struct {
    pub result: i64,
    pub publishedfileid: String,
    pub creator: String,
    pub creator_appid: i64,
    pub consumer_appid: i64,
    pub consumer_shortcutid: i64,
    pub filename: String,
    pub file_size: String,
    pub preview_file_size: String,
    pub preview_url: String,
    pub url: String,
    pub hcontent_file: String,
    pub hcontent_preview: String,
    pub title: String,
    pub file_description: String,
    pub time_created: i64,
    pub time_updated: i64,
    pub visibility: i64,
    pub flags: i64,
    pub workshop_file: bool,
    pub workshop_accepted: bool,
    pub show_subscribe_all: bool,
    pub num_comments_public: i64,
    pub banned: bool,
    pub ban_reason: String,
    pub banner: String,
    pub can_be_deleted: bool,
    pub app_name: String,
    pub file_type: i64,
    pub can_subscribe: bool,
    pub subscriptions: i64,
    pub favorited: i64,
    pub followers: i64,
    pub lifetime_subscriptions: i64,
    pub lifetime_favorited: i64,
    pub lifetime_followers: i64,
    pub lifetime_playtime: String,
    pub lifetime_playtime_sessions: String,
    pub views: i64,
    pub num_children: i64,
    pub num_reports: i64,
    pub tags: Option<Vec<Struct1>>,
    pub children: Option<Vec<Struct2>>,
    pub language: i64,
    pub maybe_inappropriate_sex: bool,
    pub maybe_inappropriate_violence: bool,
    pub revision_change_number: String,
    pub revision: i64,
    pub ban_text_check_result: i64,
    pub content_descriptorids: Option<Vec<i64>>,
}

#[derive(Serialize, Deserialize)]
struct Response {
    pub total: i64,
    pub publishedfiledetails: Vec<Struct>,
    pub next_cursor: String,
}

#[derive(Serialize, Deserialize)]
struct Root {
    pub response: Response,
}
