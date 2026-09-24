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
#[expect(clippy::missing_docs_in_private_items)]
pub struct GetTagCount {
    pub tag_id: String,
    pub app_id: u32,
}

impl GetTagCount {
    /// Builds the `GetTagCount` request
    pub fn into_request(self, client: &Client, access_token: &str) -> reqwest::Result<Request> {
        client
            .get("https://api.steampowered.com/IPublishedFileService/QueryFiles/v1/")
            .query(&[
                ("key", access_token),
                ("cursor", "*"),
                (
                    "query_type",
                    &(EPublishedFileQueryType::RankedByLastUpdatedDate as i64).to_string(),
                ),
                ("requiredtags[0]", &self.tag_id),
                ("appid", &self.app_id.to_string()),
                ("totalonly", &true.to_string()),
                ("numperpage", &1.to_string()),
            ])
            .build()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetTagCountResponse {
    pub total: i64,
}

#[expect(dead_code)]
#[expect(clippy::missing_docs_in_private_items)]
pub struct GetPage {
    pub query_type: EPublishedFileQueryType,
    pub numperpage: u32,
    pub appid: i64,
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
    #[serde(default)]
    pub previews: Vec<Preview>,
    pub preview_file_size: Option<String>,
    pub preview_url: Option<String>,
    pub publishedfileid: String,
    pub result: i32,
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
pub struct Preview {
    pub previewid: String,
    pub sortorder: i64,
    pub url: String,
    pub size: i64,
    pub filename: String,
    pub preview_type: i64,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SteamUser {
    pub steamid: String,
    pub communityvisibilitystate: i64,
    pub profilestate: Option<i64>,
    pub personaname: String,
    pub profileurl: String,
    pub avatar: String,
    pub avatarmedium: String,
    pub avatarfull: String,
    pub avatarhash: String,
    pub lastlogoff: Option<i64>,
    pub personastate: i64,
    pub realname: Option<String>,
    pub primaryclanid: Option<String>,
    pub timecreated: Option<i64>,
    pub personastateflags: Option<i64>,
    pub loccountrycode: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SteamUserResponse {
    pub players: Vec<SteamUser>,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EResult {
    /// No result. Not listed in the docs, but present in `steamclientpublic.h`.
    None = 0,
    /// Success.
    OK = 1,
    /// Generic failure.
    Fail = 2,
    /// Your Steam client doesn't have a connection to the back-end.
    NoConnection = 3,
    // 4 is unassigned.
    /// Password/ticket is invalid.
    InvalidPassword = 5,
    /// The user is logged in elsewhere.
    LoggedInElsewhere = 6,
    /// Protocol version is incorrect.
    InvalidProtocolVer = 7,
    /// A parameter is incorrect.
    InvalidParam = 8,
    /// File was not found.
    FileNotFound = 9,
    /// Called method is busy - action not taken.
    Busy = 10,
    /// Called object was in an invalid state.
    InvalidState = 11,
    /// The name was invalid.
    InvalidName = 12,
    /// The email was invalid.
    InvalidEmail = 13,
    /// The name is not unique.
    DuplicateName = 14,
    /// Access is denied.
    AccessDenied = 15,
    /// Operation timed out.
    Timeout = 16,
    /// The user is VAC2 banned.
    Banned = 17,
    /// Account not found.
    AccountNotFound = 18,
    /// The Steam ID was invalid.
    InvalidSteamID = 19,
    /// The requested service is currently unavailable.
    ServiceUnavailable = 20,
    /// The user is not logged on.
    NotLoggedOn = 21,
    /// Request is pending, it may be in process or waiting on third party.
    Pending = 22,
    /// Encryption or Decryption failed.
    EncryptionFailure = 23,
    /// Insufficient privilege.
    InsufficientPrivilege = 24,
    /// Too much of a good thing.
    LimitExceeded = 25,
    /// Access has been revoked (used for revoked guest passes).
    Revoked = 26,
    /// License/Guest pass the user is trying to access is expired.
    Expired = 27,
    /// Guest pass has already been redeemed by account, cannot be used again.
    AlreadyRedeemed = 28,
    /// The request is a duplicate and the action has already occurred in the
    /// past, ignored this time.
    DuplicateRequest = 29,
    /// All the games in this guest pass redemption request are already owned by
    /// the user.
    AlreadyOwned = 30,
    /// IP address not found.
    IPNotFound = 31,
    /// Failed to write change to the data store.
    PersistFailed = 32,
    /// Failed to acquire access lock for this operation.
    LockingFailed = 33,
    /// The logon session has been replaced.
    LogonSessionReplaced = 34,
    /// Failed to connect.
    ConnectFailed = 35,
    /// The authentication handshake has failed.
    HandshakeFailed = 36,
    /// There has been a generic IO failure.
    IOFailure = 37,
    /// The remote server has disconnected.
    RemoteDisconnect = 38,
    /// Failed to find the shopping cart requested.
    ShoppingCartNotFound = 39,
    /// A user blocked the action.
    Blocked = 40,
    /// The target is ignoring sender.
    Ignored = 41,
    /// Nothing matching the request found.
    NoMatch = 42,
    /// The account is disabled.
    AccountDisabled = 43,
    /// This service is not accepting content changes right now.
    ServiceReadOnly = 44,
    /// Account doesn't have value, so this feature isn't available.
    AccountNotFeatured = 45,
    /// Allowed to take this action, but only because requester is admin.
    AdministratorOK = 46,
    /// A version mismatch in content transmitted within the Steam protocol.
    ContentVersion = 47,
    /// The current CM can't service the user making a request, user should try
    /// another.
    TryAnotherCM = 48,
    /// You are already logged in elsewhere, this cached credential login has
    /// failed.
    PasswordRequiredToKickSession = 49,
    /// The user is logged in elsewhere. Use `LoggedInElsewhere` instead.
    AlreadyLoggedInElsewhere = 50,
    /// Long running operation has suspended/paused (e.g. content download).
    Suspended = 51,
    /// Operation has been canceled, typically by user (e.g. a content
    /// download).
    Cancelled = 52,
    /// Operation canceled because data is ill formed or unrecoverable.
    DataCorruption = 53,
    /// Operation canceled - not enough disk space.
    DiskFull = 54,
    /// The remote or IPC call has failed.
    RemoteCallFailed = 55,
    /// Password could not be verified as it's unset server side.
    PasswordUnset = 56,
    /// External account (PSN, Facebook...) is not linked to a Steam account.
    ExternalAccountUnlinked = 57,
    /// PSN ticket was invalid.
    PSNTicketInvalid = 58,
    /// External account (PSN, Facebook...) is already linked to some other
    /// account, must explicitly request to replace/delete the link first.
    ExternalAccountAlreadyLinked = 59,
    /// The sync cannot resume due to a conflict between the local and remote
    /// files.
    RemoteFileConflict = 60,
    /// The requested new password is not allowed.
    IllegalPassword = 61,
    /// New value is the same as the old one. Used for secret question and
    /// answer.
    SameAsPreviousValue = 62,
    /// Account login denied due to 2nd factor authentication failure.
    AccountLogonDenied = 63,
    /// The requested new password is not legal.
    CannotUseOldPassword = 64,
    /// Account login denied due to auth code invalid.
    InvalidLoginAuthCode = 65,
    /// Account login denied due to 2nd factor auth failure - and no mail has
    /// been sent.
    AccountLogonDeniedNoMail = 66,
    /// The user's hardware does not support Intel's Identity Protection
    /// Technology (IPT).
    HardwareNotCapableOfIPT = 67,
    /// Intel's Identity Protection Technology (IPT) has failed to initialize.
    IPTInitError = 68,
    /// Operation failed due to parental control restrictions for current user.
    ParentalControlRestricted = 69,
    /// Facebook query returned an error.
    FacebookQueryError = 70,
    /// Account login denied due to an expired auth code.
    ExpiredLoginAuthCode = 71,
    /// The login failed due to an IP restriction.
    IPLoginRestrictionFailed = 72,
    /// The current user's account is locked for use. Likely due to a hijacking
    /// and pending ownership verification.
    AccountLockedDown = 73,
    /// The logon failed because the account's email is not verified.
    AccountLogonDeniedVerifiedEmailRequired = 74,
    /// There is no URL matching the provided values.
    NoMatchingURL = 75,
    /// Bad response due to a parse failure, missing field, etc.
    BadResponse = 76,
    /// The user cannot complete the action until they re-enter their password.
    RequirePasswordReEntry = 77,
    /// The value entered is outside the acceptable range.
    ValueOutOfRange = 78,
    /// Something happened that we didn't expect to ever happen.
    UnexpectedError = 79,
    /// The requested service has been configured to be unavailable.
    Disabled = 80,
    /// The files submitted to the CEG server are not valid.
    InvalidCEGSubmission = 81,
    /// The device being used is not allowed to perform this action.
    RestrictedDevice = 82,
    /// The action could not be complete because it is region restricted.
    RegionLocked = 83,
    /// Temporary rate limit exceeded, try again later. Unlike `LimitExceeded`,
    /// which may be permanent.
    RateLimitExceeded = 84,
    /// Need two-factor code to login.
    AccountLoginDeniedNeedTwoFactor = 85,
    /// The thing we're trying to access has been deleted.
    ItemDeleted = 86,
    /// Login attempt failed, try to throttle response to possible attacker.
    AccountLoginDeniedThrottle = 87,
    /// Two factor authentication (Steam Guard) code is incorrect.
    TwoFactorCodeMismatch = 88,
    /// The activation code for two-factor authentication (Steam Guard) didn't
    /// match.
    TwoFactorActivationCodeMismatch = 89,
    /// The current account has been associated with multiple partners.
    AccountAssociatedToMultiplePartners = 90,
    /// The data has not been modified.
    NotModified = 91,
    /// The account does not have a mobile device associated with it.
    NoMobileDevice = 92,
    /// The time presented is out of range or tolerance.
    TimeNotSynced = 93,
    /// SMS code failure - no match, none pending, etc.
    SmsCodeFailed = 94,
    /// Too many accounts access this resource.
    AccountLimitExceeded = 95,
    /// Too many changes to this account.
    AccountActivityLimitExceeded = 96,
    /// Too many changes to this phone.
    PhoneActivityLimitExceeded = 97,
    /// Cannot refund to payment method, must use wallet.
    RefundToWallet = 98,
    /// Cannot send an email.
    EmailSendFailure = 99,
    /// Can't perform operation until payment has settled.
    NotSettled = 100,
    /// The user needs to provide a valid captcha.
    NeedCaptcha = 101,
    /// A game server login token owned by this token's owner has been banned.
    GSLTDenied = 102,
    /// Game server owner is denied for some other reason such as account
    /// locked, community ban, VAC ban, missing phone, etc.
    GSOwnerDenied = 103,
    /// The type of thing we were requested to act on is invalid.
    InvalidItemType = 104,
    /// The IP address has been banned from taking this action.
    IPBanned = 105,
    /// This Game Server Login Token (GSLT) has expired from disuse; it can be
    /// reset for use.
    GSLTExpired = 106,
    /// User doesn't have enough wallet funds to complete the action.
    InsufficientFunds = 107,
    /// There are too many of this thing pending already.
    TooManyPending = 108,
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
