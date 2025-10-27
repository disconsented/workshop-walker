// https://steamapi.xpaw.me/#IPublishedFileService/QueryFiles
// Gets all the items for a thing

// https://steamapi.xpaw.me/#IPublishedFileService/GetDetails
// Details for _a_ file

// https://steamcommunity.com/sharedfiles/filedetails/?id=3465175461&searchtext=
// Item URL

use std::fmt::Debug;

use reqwest::{Client, Request};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};
use snafu::Whatever;

#[expect(
    clippy::arbitrary_source_item_ordering,
    clippy::missing_docs_in_private_items
)]
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub enum EPublishedFileQueryType {
    #[default]
    RankedByVote = 0,
    RankedByPublicationDate = 1,
    AcceptedForGameRankedByAcceptanceDate = 2,
    RankedByTrend = 3,
    FavoritedByFriendsRankedByPublicationDate = 4,
    CreatedByFriendsRankedByPublicationDate = 5,
    RankedByNumTimesReported = 6,
    CreatedByFollowedUsersRankedByPublicationDate = 7,
    NotYetRated = 8,
    RankedByTotalUniqueSubscriptions = 9,
    RankedByTotalVotesAsc = 10,
    RankedByVotesUp = 11,
    RankedByTextSearch = 12,
    RankedByPlaytimeTrend = 13,
    RankedByTotalPlaytime = 14,
    RankedByAveragePlaytimeTrend = 15,
    RankedByLifetimeAveragePlaytime = 16,
    RankedByPlaytimeSessionsTrend = 17,
    RankedByLifetimePlaytimeSessions = 18,
    RankedByInappropriateContentRating = 19,
    RankedByBanContentCheck = 20,
    RankedByLastUpdatedDate = 21,
}

#[expect(non_camel_case_types, clippy::missing_docs_in_private_items)] // Can't control the _ and steam requires it
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone)]
#[repr(u8)]
pub enum EPublishedFileInfoMatchingFileType {
    MatchingFileType_Items = 0,
    MatchingFileType_Collections = 1,
    MatchingFileType_Art = 2,
    MatchingFileType_Videos = 3,
    MatchingFileType_Screenshots = 4,
    MatchingFileType_CollectionEligible = 5,
    MatchingFileType_Games = 6,
    MatchingFileType_Software = 7,
    MatchingFileType_Concepts = 8,
    MatchingFileType_GreenlightItems = 9,
    MatchingFileType_AllGuides = 10,
    MatchingFileType_WebGuides = 11,
    MatchingFileType_IntegratedGuides = 12,
    MatchingFileType_UsableInGame = 13,
    MatchingFileType_Merch = 14,
    MatchingFileType_ControllerBindings = 15,
    MatchingFileType_SteamworksAccessInvites = 16,
    MatchingFileType_Items_Mtx = 17,
    MatchingFileType_Items_ReadyToUse = 18,
    MatchingFileType_WorkshopShowcase = 19,
    MatchingFileType_GameManagedItems = 20,
}

#[expect(dead_code)]
#[expect(clippy::missing_docs_in_private_items)]
pub struct GetPage {
    pub query_type: EPublishedFileQueryType,
    pub numperpage: u32,
    pub appid: u32,
    pub return_tags: bool,
    pub return_children: bool,
    pub return_details: bool,
    pub return_metadata: bool,
    pub return_previews: bool,
    pub return_vote_data: bool,
    pub return_short_description: bool,
    pub strip_description_bbcode: bool,
    pub admin_query: bool,
    pub cursor: String,
}

impl Default for GetPage {
    fn default() -> Self {
        Self {
            query_type: EPublishedFileQueryType::RankedByLastUpdatedDate,
            numperpage: 100,
            appid: 0,
            return_tags: true,
            return_children: true,
            return_details: true,
            return_metadata: true,
            return_previews: true,
            return_vote_data: true,
            return_short_description: true,
            strip_description_bbcode: false,
            admin_query: true,
            cursor: "*".to_string(),
        }
    }
}

impl GetPage {
    /// Builds the `GetPage` request
    pub fn into_request(self, client: &Client, access_token: &str) -> reqwest::Result<Request> {
        client
            .get("https://api.steampowered.com/IPublishedFileService/QueryFiles/v1/")
            .query(&[
                ("key", access_token),
                ("cursor", &self.cursor),
                ("numperpage", &self.numperpage.to_string()),
                ("appid", &self.appid.to_string()),
                ("return_tags", &self.return_tags.to_string()),
                ("return_vote_data", &self.return_vote_data.to_string()),
                ("return_children", &self.return_children.to_string()),
                ("return_details", &self.return_details.to_string()),
                (
                    "strip_description_bbcode",
                    &self.strip_description_bbcode.to_string(),
                ),
            ])
            .build()
    }
}

impl TryFrom<&SteamRoot<IPublishedResponse>> for GetPage {
    type Error = Whatever;

    fn try_from(value: &SteamRoot<IPublishedResponse>) -> Result<Self, Self::Error> {
        Ok(GetPage {
            cursor: value.response.next_cursor.clone(),
            ..Default::default()
        })
    }
}
#[expect(clippy::missing_docs_in_private_items)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Child {
    pub publishedfileid: String,
    pub sortorder: i64,
    pub file_type: i64,
}
#[expect(clippy::missing_docs_in_private_items)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tag {
    pub tag: String,
    pub display_name: String,
}

#[expect(
    clippy::missing_docs_in_private_items,
    reason = "Largely unused, exists for serde's sake"
)]
#[expect(clippy::struct_excessive_bools, reason = "Steam defined")]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IPublishedStruct {
    pub file_type: Option<EPublishedFileInfoMatchingFileType>,
    pub app_name: Option<String>,
    pub ban_reason: Option<String>,
    pub ban_text_check_result: Option<i64>,
    #[serde(default)]
    pub banned: bool,
    pub banner: Option<String>,
    #[serde(default)]
    pub can_be_deleted: bool,
    #[serde(default)]
    pub can_subscribe: bool,
    #[serde(default)]
    pub children: Vec<Child>,
    pub consumer_appid: Option<i64>,
    pub consumer_shortcutid: Option<i64>,
    pub content_descriptorids: Option<Vec<i64>>,
    pub creator: Option<String>,
    pub creator_appid: Option<i64>,
    pub favorited: Option<i64>,
    pub file_description: Option<String>,
    pub file_size: Option<String>,
    pub filename: Option<String>,
    pub flags: Option<i64>,
    pub followers: Option<i64>,
    pub hcontent_file: Option<String>,
    pub hcontent_preview: Option<String>,
    pub language: Option<i64>,
    pub lifetime_favorited: Option<i64>,
    pub lifetime_followers: Option<i64>,
    pub lifetime_playtime: Option<String>,
    pub lifetime_playtime_sessions: Option<String>,
    pub lifetime_subscriptions: Option<i64>,
    #[serde(default)]
    pub maybe_inappropriate_sex: bool,
    #[serde(default)]
    pub maybe_inappropriate_violence: bool,
    pub num_children: Option<i64>,
    pub num_comments_public: Option<i64>,
    pub num_reports: Option<i64>,
    pub preview_file_size: Option<String>,
    pub preview_url: Option<String>,
    pub publishedfileid: String,
    pub result: i64,
    pub revision: Option<i64>,
    pub revision_change_number: Option<String>,
    #[serde(default)]
    pub show_subscribe_all: bool,
    pub subscriptions: Option<i64>,
    #[serde(default)]
    pub tags: Vec<Tag>,
    pub time_created: Option<i64>,
    pub time_updated: Option<i64>,
    pub title: Option<String>,
    pub url: Option<String>,
    pub views: Option<i64>,
    pub visibility: Option<i64>,
    pub vote_data: Option<VoteData>,
    #[serde(default)]
    pub workshop_accepted: bool,
    #[serde(default)]
    pub workshop_file: bool,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IPublishedResponse {
    pub total: i64,
    #[serde(default)]
    pub publishedfiledetails: Vec<Value>,
    pub next_cursor: String,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SteamRoot<T: Clone + Debug> {
    pub response: T,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VoteData {
    pub score: f32,
    pub votes_up: usize,
    pub votes_down: usize,
}

#[cfg(test)]
mod test {
    use std::fs::read_to_string;

    use crate::steam::model::{IPublishedResponse, SteamRoot};

    #[test]
    fn test_parse_all() {
        let txt = read_to_string("./src/steam/test_all.json").unwrap();
        let data: SteamRoot<IPublishedResponse> = serde_json::from_str(&txt).unwrap();
        dbg!(data);
    }

    #[test]
    fn test_parse_1() {
        let txt = read_to_string("./src/steam/test_1.json").unwrap();
        let data: SteamRoot<IPublishedResponse> = serde_json::from_str(&txt).unwrap();
        dbg!(data);
    }

    #[test]
    fn test_parse_bad() {
        let txt = read_to_string("./src/steam/dead.json").unwrap();
        let data: SteamRoot<IPublishedResponse> = serde_json::from_str(&txt).unwrap();
        dbg!(data);
    }
}
