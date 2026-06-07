use alloc::alloc;
use std::fmt::{Display, Formatter};

use chrono::{DateTime, Utc};
use macros::dual_struct;
use salvo::prelude::ToSchema;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use serde_content::{Value, ValueVisitor};
use serde_hack::ValueRefDeserializer;
use serde_repr::{Deserialize_repr, Serialize_repr};
use surrealdb_types::{Number, Object, RecordIdKey, SurrealValue};
use tracing::{error, warn};

use crate::{
    db::{AppID, IAppID, IItemID, ITagID, IUserID, IUsernameID, ItemID, TagID, UserID, UsernameID},
    processing::language_actor::DetectedLanguage,
};

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, Default)]
pub enum OrderBy {
    Alphabetical,
    #[default]
    LastUpdated,
}

impl OrderBy {
    pub fn column_name(&self) -> &str {
        match self {
            OrderBy::Alphabetical => "title",
            OrderBy::LastUpdated => "last_updated",
        }
    }
}

impl Display for OrderBy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[dual_struct(derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Hash,
    Ord,
    PartialOrd
))]
pub struct Tag {
    #[dual_type(ITagID)]
    pub id: TagID,
    pub display_name: String,
}

fn to_external_tag(internal: Vec<InternalTag>) -> Result<Vec<ExternalTag>, surrealdb_types::Error> {
    internal
        .into_iter()
        .map(ExternalTag::try_from)
        .collect::<Result<_, _>>()
        .inspect_err(|error| error!(?error, "to_external_tag"))
}

fn to_internal_tag(external: Vec<ExternalTag>) -> Vec<InternalTag> {
    external.into_iter().map(InternalTag::from).collect()
}

#[dual_struct(derive(Serialize, Deserialize, Clone, Debug))]
pub struct WorkshopItem {
    #[dual_type(IAppID)]
    pub app: AppID,
    #[dual_type(IUsernameID)]
    pub author: UsernameID,
    pub description: String,
    #[dual_type(IItemID)]
    pub id: ItemID,
    pub languages: Vec<DetectedLanguage>,
    pub last_updated: u64,
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
}

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
    pub preview_url: Option<String>,
    pub title: String,
    pub score: f32,
    pub tags: Vec<ITagID>,
}
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
}

fn to_external_username(
    internal: Option<InternalUsername>,
) -> Result<Option<ExternalUsername>, surrealdb_types::Error> {
    internal.map(TryInto::try_into).transpose()
}

fn to_internal_username(external: Option<ExternalUsername>) -> Option<InternalUsername> {
    external.map(Into::into)
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
    #[dual_type(Vec<InternalTag>, to_external = to_external_tag, to_internal = to_internal_tag)]
    pub default_tags: Vec<ExternalTag>,
    /// List of known tags
    #[serde(default)]
    #[dual_type(Vec<InternalTag>, to_external = to_external_tag, to_internal = to_internal_tag)]
    pub tags: Vec<ExternalTag>,
}

/// A workshop walker user
#[dual_struct(derive(Serialize, Deserialize, Clone, Debug))]
pub struct User {
    /// The steam account ID
    #[dual_type(IUserID)]
    pub id: UserID,
    /// Privileged access
    pub admin: bool,
    pub banned: bool,
    /// UTC timestamp of when the user last logged in
    // Surrealdb bug: https://github.com/surrealdb/surrealdb/issues/3550
    #[serde(serialize_with = "serialize_chrono_as_sql_datetime")]
    pub last_logged_in: DateTime<Utc>,
}
pub fn serialize_chrono_as_sql_datetime<S>(x: &DateTime<Utc>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    Into::<surrealdb_types::Datetime>::into(*x).serialize(s)
}

/// Crowdsourced metadata for an item, private version
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    ToSchema,
    Eq,
    PartialEq,
    Hash,
    SurrealValue,
    Ord,
    PartialOrd,
)]
pub struct Property {
    pub class: Class,
    pub value: String,
}

impl Display for Property {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.class.to_string())?;
        f.write_str(":")?;
        f.write_str(&self.value)
    }
}

impl Into<RecordIdKey> for Property {
    fn into(self) -> RecordIdKey {
        let mut obj = Object::new();
        obj.insert("class", self.class);
        obj.insert("value", self.value);
        RecordIdKey::Object(obj)
    }
}

#[dual_struct(derive(Serialize, Deserialize, Clone, Debug))]
pub struct WorkshopItemProperties {
    #[dual_type(IItemID, surreal(wrap))]
    #[serde(rename = "in")]
    pub r#in: ItemID,
    #[serde(rename = "out")]
    pub out: Property,
    /// Reasoning or justification for an inclusion
    pub note: Option<String>,
    pub status: Status,
    /// The current score
    pub upvote_count: i64,
    /// The total upvotes
    pub vote_count: u64,
    #[dual_type(InternalSource)]
    pub source: ExternalSource,
    pub vote_state: Option<i32>,
}

fn to_external_props(
    internal: Vec<InternalWorkshopItemProperties>,
) -> Result<Vec<ExternalWorkshopItemProperties>, surrealdb_types::Error> {
    internal
        .into_iter()
        .map(TryFrom::try_from)
        .collect::<Result<_, _>>()
}

fn to_internal_props(
    external: Vec<ExternalWorkshopItemProperties>,
) -> Vec<InternalWorkshopItemProperties> {
    external.into_iter().map(From::from).collect()
}

/// Crowdsourced relationships for an item, used for "soft" dependencies not
/// supplied by steam, private version
// #[expect(unused, reason = "To be used soon")]
// #[derive(Serialize, Deserialize, Clone, Debug, ToSchema, SurrealValue)]
// pub struct Companion<R, S> {
//     /// Snowflake generated ID
//     pub id: String,
//     pub r#in: R,
//     pub out: R,
//     /// Reasoning or justification for an inclusion
//     pub note: Option<String>,
//     pub status: Status,
//     pub upvote_count: u64,
//     pub vote_count: u64,
//     pub source: Source<S>,
// }

/// A voting record
#[dual_struct(derive(Serialize, Deserialize, Clone, Debug))]
pub struct Vote {
    /// The app this is associated with, for possible filtering
    #[dual_type(IAppID)]
    pub app: AppID,
    pub score: f32,
    pub when: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InternalSource {
    /// Auto-generated
    System,
    /// User submitted
    User(IUserID),
}

impl serde::Serialize for InternalSource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            InternalSource::System => serializer.serialize_str("system"),
            InternalSource::User(t) => t.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for InternalSource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = <Value as serde::Deserialize>::deserialize(deserializer)?;
        let deserializer = ValueRefDeserializer::<D::Error>::new(&value);
        let value = deserializer.deserialize_any(ValueVisitor)?;

        match value {
            Value::String(str) if str == "system" => Ok(InternalSource::System),
            _ => <IUserID as serde::Deserialize>::deserialize(deserializer)
                .map(InternalSource::User)
                .map_err(|_| {
                    Error::custom("data did not match any variant of untagged enum Source")
                }),
        }
    }
}

impl From<ExternalSource> for InternalSource {
    fn from(value: ExternalSource) -> Self {
        match value {
            ExternalSource::System => Self::System,
            ExternalSource::User(t) => Self::User(t.into()),
        }
    }
}

impl SurrealValue for InternalSource {
    fn into_value(self) -> ::surrealdb::types::Value {
        match self {
            Self::System => {
                let mut map = ::surrealdb::types::Object::new();
                map.insert(
                    "System".to_string(),
                    ::surrealdb::types::Value::Object(::surrealdb::types::Object::new()),
                );
                ::surrealdb::types::Value::Object(map)
            }
            Self::User(field_0) => {
                let mut map = ::surrealdb::types::Object::new();
                map.insert(
                    "User".to_string(),
                    ::surrealdb::types::Value::from_t(field_0),
                );
                ::surrealdb::types::Value::Object(map)
            }
        }
    }

    fn from_value(
        value: ::surrealdb::types::Value,
    ) -> std::result::Result<Self, ::surrealdb::types::Error> {
        match value {
            ::surrealdb::types::Value::Object(mut map) => {
                {
                    if map.get("System").is_some() {
                        return Ok(Self::System);
                    }
                }
                {
                    if let Some(value) = map.remove("User") {
                        {
                            let field_0 = <IUserID as SurrealValue>::from_value(value)?;
                            return Ok(Self::User(field_0));
                        }
                    }
                }
            },
            ::surrealdb::types::Value::String(string) => {
                if string == "system" {
                    return Ok(Self::System);
                }
            }
            _ => {}
        };
        Err(::surrealdb::types::Error::internal(format!(
            "Failed to decode {}, no variants matched",
            "InternalSource"
        )))
    }

    fn is_value(value: &::surrealdb::types::Value) -> bool {
        match value {
            ::surrealdb::types::Value::Object(map) => {
                {
                    if map
                        .get("System")
                        .is_some_and(|v| v.is_object_and(|o| o.is_empty()))
                    {
                        return true;
                    }
                }
                {
                    if let Some(value) = map.get("User") {
                        return <IUserID as SurrealValue>::is_value(value);
                    }
                }
            }
            _ => {}
        }
        false
    }

    fn kind_of() -> ::surrealdb::types::Kind {
        ::surrealdb::types::Kind::Either(vec![
            {
                let mut obj = std::collections::BTreeMap::new();
                obj.insert(
                    "System".to_string(),
                    ::surrealdb::types::Kind::Literal(::surrealdb::types::KindLiteral::Object(
                        std::collections::BTreeMap::new(),
                    )),
                );
                ::surrealdb::types::Kind::Literal(::surrealdb::types::KindLiteral::Object(obj))
            },
            {
                let mut obj = std::collections::BTreeMap::new();
                obj.insert("User".to_string(), <IUserID as SurrealValue>::kind_of());
                ::surrealdb::types::Kind::Literal(::surrealdb::types::KindLiteral::Object(obj))
            },
        ])
    }
}

#[derive(Clone, Debug, Eq, PartialEq, ToSchema, Serialize, Deserialize)]
pub enum ExternalSource {
    /// Auto-generated
    System,
    /// User submitted
    User(UserID),
}

impl TryFrom<InternalSource> for ExternalSource {
    type Error = surrealdb_types::Error;

    fn try_from(value: InternalSource) -> Result<Self, Self::Error> {
        match value {
            InternalSource::System => Ok(ExternalSource::System),
            InternalSource::User(t) => Ok(ExternalSource::User(t.try_into()?)),
        }
    }
}

#[derive(
    Debug,
    ToSchema,
    Clone,
    Serialize,
    Deserialize,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    /* SurrealValue, */
)]
pub enum Class {
    /// Anything like addon, overhaul, bugfix, patch
    Type,
    /// Literary themes like mecha
    Theme,
    /// Literary genres like `CyberPunk`
    Genre,
    /// Mod features, like "new scenario" or "new clothes"
    Feature,
}

impl SurrealValue for Class {
    fn into_value(self) -> ::surrealdb::types::Value {
        match self {
            Self::Type => {
                let mut map = ::surrealdb::types::Object::new();
                map.insert(
                    "Type".to_string(),
                    ::surrealdb::types::Value::Object(::surrealdb::types::Object::new()),
                );
                ::surrealdb::types::Value::Object(map)
            }
            Self::Theme => {
                let mut map = ::surrealdb::types::Object::new();
                map.insert(
                    "Theme".to_string(),
                    ::surrealdb::types::Value::Object(::surrealdb::types::Object::new()),
                );
                ::surrealdb::types::Value::Object(map)
            }
            Self::Genre => {
                let mut map = ::surrealdb::types::Object::new();
                map.insert(
                    "Genre".to_string(),
                    ::surrealdb::types::Value::Object(::surrealdb::types::Object::new()),
                );
                ::surrealdb::types::Value::Object(map)
            }
            Self::Feature => {
                let mut map = ::surrealdb::types::Object::new();
                map.insert(
                    "Feature".to_string(),
                    ::surrealdb::types::Value::Object(::surrealdb::types::Object::new()),
                );
                ::surrealdb::types::Value::Object(map)
            }
        }
    }

    fn from_value(
        value: ::surrealdb::types::Value,
    ) -> std::result::Result<Self, ::surrealdb::types::Error> {
        match value {
            ::surrealdb::types::Value::Object(mut map) => {
                {
                    if map.get("Type").is_some() {
                        return Ok(Self::Type);
                    }
                }
                {
                    if map.get("Theme").is_some() {
                        return Ok(Self::Theme);
                    }
                }
                {
                    if map.get("Genre").is_some() {
                        return Ok(Self::Genre);
                    }
                }
                {
                    if map.get("Feature").is_some() {
                        return Ok(Self::Feature);
                    }
                }
            }
            ::surrealdb::types::Value::String(value) => match value.as_str() {
                "Type" => return Ok(Self::Type),
                "Theme" => return Ok(Self::Theme),
                "Genre" => return Ok(Self::Genre),
                "Feature" => return Ok(Self::Feature),
                _ => {}
            },
            _ => {}
        };
        Err(::surrealdb::types::Error::internal(format!(
            "Failed to decode {}, no variants matched",
            "Class"
        )))
    }

    fn is_value(value: &::surrealdb::types::Value) -> bool {
        match value {
            ::surrealdb::types::Value::Object(map) => {
                {
                    if map
                        .get("Type")
                        .is_some_and(|v| v.is_object_and(|o| o.is_empty()))
                    {
                        return true;
                    }
                }
                {
                    if map
                        .get("Theme")
                        .is_some_and(|v| v.is_object_and(|o| o.is_empty()))
                    {
                        return true;
                    }
                }
                {
                    if map
                        .get("Genre")
                        .is_some_and(|v| v.is_object_and(|o| o.is_empty()))
                    {
                        return true;
                    }
                }
                {
                    if map
                        .get("Feature")
                        .is_some_and(|v| v.is_object_and(|o| o.is_empty()))
                    {
                        return true;
                    }
                }
            }
            _ => {}
        }
        false
    }

    fn kind_of() -> ::surrealdb::types::Kind {
        ::surrealdb::types::Kind::Either(vec![
            {
                let mut obj = std::collections::BTreeMap::new();
                obj.insert(
                    "Type".to_string(),
                    ::surrealdb::types::Kind::Literal(::surrealdb::types::KindLiteral::Object(
                        std::collections::BTreeMap::new(),
                    )),
                );

                ::surrealdb::types::Kind::Literal(::surrealdb::types::KindLiteral::Object(obj))
            },
            {
                let mut obj = std::collections::BTreeMap::new();
                obj.insert(
                    "Theme".to_string(),
                    ::surrealdb::types::Kind::Literal(::surrealdb::types::KindLiteral::Object(
                        std::collections::BTreeMap::new(),
                    )),
                );

                ::surrealdb::types::Kind::Literal(::surrealdb::types::KindLiteral::Object(obj))
            },
            {
                let mut obj = std::collections::BTreeMap::new();
                obj.insert(
                    "Genre".to_string(),
                    ::surrealdb::types::Kind::Literal(::surrealdb::types::KindLiteral::Object(
                        std::collections::BTreeMap::new(),
                    )),
                );

                ::surrealdb::types::Kind::Literal(::surrealdb::types::KindLiteral::Object(obj))
            },
            {
                let mut obj = std::collections::BTreeMap::new();
                obj.insert(
                    "Feature".to_string(),
                    ::surrealdb::types::Kind::Literal(::surrealdb::types::KindLiteral::Object(
                        std::collections::BTreeMap::new(),
                    )),
                );

                ::surrealdb::types::Kind::Literal(::surrealdb::types::KindLiteral::Object(obj))
            },
        ])
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let txt = match self {
            Class::Type => "TYPE",
            Class::Theme => "THEME",
            Class::Genre => "GENRE",
            Class::Feature => "FEATURE",
        };
        f.write_str(txt)
    }
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
// Horrendous manual hack thanks to _another_ surreal bug
impl SurrealValue for Status {
    fn into_value(self) -> ::surrealdb::types::Value {
        match self {
            Self::Rejected => {
                let mut map = ::surrealdb::types::Object::new();
                map.insert(
                    "Rejected".to_string(),
                    ::surrealdb::types::Value::Object(::surrealdb::types::Object::new()),
                );
                ::surrealdb::types::Value::Object(map)
            }
            Self::Pending => {
                let mut map = ::surrealdb::types::Object::new();
                map.insert(
                    "Pending".to_string(),
                    ::surrealdb::types::Value::Object(::surrealdb::types::Object::new()),
                );
                ::surrealdb::types::Value::Object(map)
            }
            Self::Accepted => {
                let mut map = ::surrealdb::types::Object::new();
                map.insert(
                    "Accepted".to_string(),
                    ::surrealdb::types::Value::Object(::surrealdb::types::Object::new()),
                );
                ::surrealdb::types::Value::Object(map)
            }
        }
    }

    fn from_value(
        value: ::surrealdb::types::Value,
    ) -> std::result::Result<Self, ::surrealdb::types::Error> {
        match value {
            ::surrealdb::types::Value::Object(mut map) => {
                {
                    if map.get("Rejected").is_some() {
                        return Ok(Self::Rejected);
                    }
                }
                {
                    if map.get("Pending").is_some() {
                        return Ok(Self::Pending);
                    }
                }
                {
                    if map.get("Accepted").is_some() {
                        return Ok(Self::Accepted);
                    }
                }
            }
            ::surrealdb::types::Value::Number(number) => match number {
                Number::Int(-1) => {
                    return Ok(Self::Rejected);
                }
                Number::Int(0) => {
                    return Ok(Self::Pending);
                }
                Number::Int(1) => {
                    return Ok(Self::Accepted);
                }
                _ => {}
            },
            _ => {}
        };
        Err(::surrealdb::types::Error::internal(format!(
            "Failed to decode {}, no variants matched",
            "Status"
        )))
    }

    fn is_value(value: &::surrealdb::types::Value) -> bool {
        warn!("{:?}", value);
        match dbg!(value) {
            ::surrealdb::types::Value::Object(map) => {
                {
                    if map
                        .get("Rejected")
                        .is_some_and(|v| v.is_object_and(|o| o.is_empty()))
                    {
                        return true;
                    }
                }
                {
                    if map
                        .get("Pending")
                        .is_some_and(|v| v.is_object_and(|o| o.is_empty()))
                    {
                        return true;
                    }
                }
                {
                    if map
                        .get("Accepted")
                        .is_some_and(|v| v.is_object_and(|o| o.is_empty()))
                    {
                        return true;
                    }
                }
            }
            _ => {}
        }
        false
    }

    fn kind_of() -> ::surrealdb::types::Kind {
        ::surrealdb::types::Kind::Either(vec![
            {
                let mut obj = std::collections::BTreeMap::new();
                obj.insert(
                    "Rejected".to_string(),
                    ::surrealdb::types::Kind::Literal(::surrealdb::types::KindLiteral::Object(
                        std::collections::BTreeMap::new(),
                    )),
                );
                ::surrealdb::types::Kind::Literal(::surrealdb::types::KindLiteral::Object(obj))
            },
            {
                let mut obj = std::collections::BTreeMap::new();
                obj.insert(
                    "Pending".to_string(),
                    ::surrealdb::types::Kind::Literal(::surrealdb::types::KindLiteral::Object(
                        std::collections::BTreeMap::new(),
                    )),
                );
                ::surrealdb::types::Kind::Literal(::surrealdb::types::KindLiteral::Object(obj))
            },
            {
                let mut obj = std::collections::BTreeMap::new();
                obj.insert(
                    "Accepted".to_string(),
                    ::surrealdb::types::Kind::Literal(::surrealdb::types::KindLiteral::Object(
                        std::collections::BTreeMap::new(),
                    )),
                );
                ::surrealdb::types::Kind::Literal(::surrealdb::types::KindLiteral::Object(obj))
            },
        ])
    }
}

#[dual_struct(derive(Serialize, Deserialize, Clone, Debug))]
pub struct Username {
    #[dual_type(IUsernameID)]
    pub id: UsernameID,
    pub name: String,
}

#[cfg(test)]
mod test {

    // use crate::db::{
    //     model::{Class, Id, InternalSource, Source}, IItemID,
    //     IUserID,
    // };
    //
    // #[test]
    // fn test_id_newtype() {
    //     let id: Id = IItemID::from("1".to_string()).into();
    //     let id_txt = serde_json::to_string(&id).unwrap();
    //     let id2: Id = serde_json::from_str(&id_txt).unwrap();
    //     assert_eq!(id, id2);
    //
    //     println!("{id_txt}");
    // }
    //
    // #[test]
    // fn test_source_de_ser() {
    //     let system: InternalSource = Source::System;
    //     let system_text = serde_json::to_string(&system).unwrap();
    //     let system2 = serde_json::from_str(&system_text).unwrap();
    //     assert_eq!(system, system2);
    //
    //     let user = Source::User("a".to_string());
    //     let user_text = serde_json::to_string(&user).unwrap();
    //     let user2 = serde_json::from_str(&user_text).unwrap();
    //     assert_eq!(user, user2);
    //     println!("{user_text} {system_text}");
    //
    //     {
    //         let user = Source::User(IUserID::from("b".to_string()));
    //         let user_text = serde_json::to_string(&user).unwrap();
    //         let user2 = serde_json::from_str(&user_text).unwrap();
    //         assert_eq!(user, user2);
    //         println!("{user_text} {system_text}");
    //     }
    //
    //     #[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
    //     struct Test {
    //         source: InternalSource,
    //     }
    //
    //     let t_user = Test { source: user };
    //     let txt_user = serde_json::to_string(&t_user).unwrap();
    //     assert_eq!(t_user, serde_json::from_str(&txt_user).unwrap());
    //     let t_sys = Test { source: system };
    //     let txt_sys = serde_json::to_string(&t_sys).unwrap();
    //     assert_eq!(t_sys, serde_json::from_str(&txt_sys).unwrap());
    //     println!("{txt_user} {txt_sys}");
    //     #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
    //     struct Foo {
    //         thing: Class,
    //     }
    //     println!(
    //         "{}",
    //         serde_json::to_string(&Foo {
    //             thing: Class::Genre
    //         })
    //         .unwrap()
    //     );
    // }
    // #[tokio::test]
    // async fn test_source_surreal() {
    //     use surrealdb::{engine::local::Mem, Surreal};
    //
    //     #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
    //     struct Foo {
    //         source: Source<RecordId>,
    //     }
    //     let db = Surreal::new::<Mem>(()).await.unwrap();
    //     db.use_ns("test").use_db("test").await.unwrap();
    //     db.query("DEFINE TABLE OVERWRITE properties TYPE NORMAL SCHEMAFULL
    // PERMISSIONS NONE;")         .query(
    //             "DEFINE FIELD OVERWRITE source ON properties TYPE 'system' |
    // record<users> \              PERMISSIONS FULL;",
    //         )
    //         .await
    //         .unwrap();
    //     let foo_struct = Foo {
    //         source: Source::System,
    //     };
    //     let mut r: Vec<Foo> = db
    //         .insert("properties")
    //         .content(foo_struct.clone())
    //         .await
    //         .unwrap();
    //     assert_eq!(foo_struct, r.pop().unwrap());
    // }
}
