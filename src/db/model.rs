use std::{
    fmt::{Display, Formatter},
    ops::{Deref, DerefMut},
};

use chrono::{DateTime, Utc};
use salvo::prelude::ToSchema;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use serde_content::{Value, ValueVisitor};
use serde_hack::ValueRefDeserializer;
use serde_repr::{Deserialize_repr, Serialize_repr};
use surrealdb_types::{RecordId, SurrealValue};
use macros::dual_struct;
use crate::db::{AppID, IAppID, IItemDependencyID, IItemID, ITagID, ItemID, TagID, UserID};
use crate::processing::language_actor::DetectedLanguage;
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

#[dual_struct(derive(Serialize, Deserialize, Clone, Debug))]
pub struct Tag {
    #[dual_type(IAppID)]
    pub app_id: AppID,
    #[dual_type(ITagID)]
    pub id: TagID,
}

#[dual_struct(derive(Serialize, Deserialize, Clone, Debug))]
pub struct WorkshopItem {
    #[dual_type(IAppID)]
    pub app_id: AppID,
    pub author: String,
    pub description: String,
    #[dual_type(IItemID)]
    pub id: ItemID,
    // pub languages: Vec<DetectedLanguage>,
    pub last_updated: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_url: Option<String>,
    pub title: String,
    #[dual_type(Vec<InternalTag>)]
    pub tags: Vec<ExternalTag>,
    pub score: f32,
    pub properties: Vec<WorkshopItemProperties<String, Property>>,
}
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct FullWorkshopItem {
    // Core identifiers
    pub id: ItemID,   // The item's ID
    pub appid: AppID, // The steam ID of the app this belongs to

    // Content information
    pub title: String,       // The titles name
    pub description: String, // HTML encoded description from steam
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview_url: Option<String>, // The URL to the banner image

    // Metadata and categorization
    #[serde(default)]
    pub tags: Vec<Tag>, // The list of tags found
    #[serde(default)]
    pub properties: Vec<WorkshopItemProperties<ItemID, Property>>, // Approved or owned properties
    pub score: f32, // The "quality" score assigned by steam

    // Author and timing
    pub author: Option<DisplayUser>, // Authors steam ID
    pub last_updated: u64,           // Timestamp in milliseconds

    // Localization
    #[serde(default)]
    pub languages: Vec<DetectedLanguage>, // All languages found in the items description

    // Dependencies
    #[serde(default)]
    pub dependencies: Vec<InternalFullWorkshopItem>, // A list of dependencies found
    #[serde(default)]
    pub dependants: Vec<InternalFullWorkshopItem>, // A list of dependants found
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Dependencies {
    pub id: IItemDependencyID,
    #[serde(rename = "in")]
    pub this: IItemID,
    #[serde(rename = "out")]
    pub dependency: IItemID,
}
/// A steam workshop app
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, SurrealValue)]
pub struct App<T> {
    /// The steam ID for an app
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
    #[salvo(schema(skip))]
    #[serde(default)]
    pub default_tags: Vec<T>,
    /// List of known tags
    #[salvo(schema(skip))]
    #[serde(default)]
    pub tags: Vec<T>,
}

/// A workshop walker user
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, SurrealValue)]
pub struct User<T> {
    /// The steam account ID
    pub id: T,
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
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, Eq, PartialEq, Hash, SurrealValue)]
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
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, SurrealValue)]
pub struct PropertyExt<SOURCE> {
    /// Reasoning or justification for an inclusion
    pub note: Option<String>,
    pub status: Status,
    /// The current score
    pub upvote_count: i64,
    /// The total upvotes
    pub vote_count: u64,
    pub source: Source<SOURCE>,
}
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, SurrealValue)]
pub struct WorkshopItemProperties<CHILD, PROP> {
    #[serde(rename = "in")]
    pub workshop_item: CHILD,
    #[serde(rename = "out")]
    pub property: PROP,
    #[serde(flatten)]
    pub property_ext: PropertyExt<CHILD>,
    pub vote_state: Option<i32>,
}

/// Crowdsourced relationships for an item, used for "soft" dependencies not
/// supplied by steam, private version
#[expect(unused, reason = "To be used soon")]
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, SurrealValue)]
pub struct Companion<R, S> {
    /// Snowflake generated ID
    pub id: String,
    pub r#in: R,
    pub out: R,
    /// Reasoning or justification for an inclusion
    pub note: Option<String>,
    pub status: Status,
    pub upvote_count: u64,
    pub vote_count: u64,
    pub source: Source<S>,
}

/// A voting record
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, SurrealValue)]
pub struct Vote {
    /// The app this is associated with, for possible filtering
    pub app_id: AppID,
    pub score: f32,
    pub when: DateTime<Utc>,
}

#[derive(Clone, Debug, ToSchema, Eq, PartialEq, SurrealValue)]
pub enum Source<T> {
    /// Auto-generated
    System,
    /// User submitted
    User(T),
}

impl<T> serde::Serialize for Source<T>
where
    T: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Source::System => serializer.serialize_str("system"),
            Source::User(t) => t.serialize(serializer),
        }
    }
}

impl<'de, T> serde::Deserialize<'de> for Source<T>
where
    T: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = <Value as serde::Deserialize>::deserialize(deserializer)?;
        let deserializer = ValueRefDeserializer::<D::Error>::new(&value);
        let value = deserializer.deserialize_any(ValueVisitor)?;

        match value {
            Value::String(str) if str == "system" => Ok(Source::System),
            _ => <T as serde::Deserialize>::deserialize(deserializer)
                .map(Source::User)
                .map_err(|_| {
                    Error::custom("data did not match any variant of untagged enum Source")
                }),
        }
    }
}

#[derive(Debug, ToSchema, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash, SurrealValue)]
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
    SurrealValue,
)]
#[repr(i8)]
pub enum Status {
    Rejected = -1,
    #[default]
    Pending = 0,
    Accepted = 1,
}

#[derive(Debug, ToSchema, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DisplayUser {
    id: UserID,
    name: String,
}
#[derive(Serialize, Deserialize, PartialOrd, PartialEq, Eq, Debug)]
#[serde(transparent)]
struct Id(RecordId);

impl From<RecordId> for Id {
    fn from(value: RecordId) -> Self {
        Self(value)
    }
}
impl From<Id> for RecordId {
    fn from(val: Id) -> Self {
        val.0
    }
}

impl Deref for Id {
    type Target = RecordId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Id {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl salvo::oapi::ToSchema for Id {
    fn to_schema(
        components: &mut salvo::oapi::Components,
    ) -> salvo::oapi::RefOr<salvo::oapi::schema::Schema> {
        let name = salvo::oapi::naming::assign_name::<Id>(salvo::oapi::naming::NameRule::Auto);
        let ref_or = salvo::oapi::RefOr::Ref(salvo::oapi::Ref::new(format!(
            "#/components/schemas/{name}"
        )));
        if !components.schemas.contains_key(&name) {
            components.schemas.insert(name.clone(), ref_or.clone());
            let schema = salvo::oapi::Object::new().schema_type(
                salvo::oapi::schema::SchemaType::basic(salvo::oapi::schema::BasicType::String),
            );
            components.schemas.insert(name, schema);
        }
        ref_or
    }
}

#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};
    use surrealdb_types::RecordId;
    use crate::db::{
        model::{Class, Id, Source}, IItemID,
        IUserID,
    };

    #[test]
    fn test_id_newtype() {
        let id: Id = IItemID::from("1".to_string()).into();
        let id_txt = serde_json::to_string(&id).unwrap();
        let id2: Id = serde_json::from_str(&id_txt).unwrap();
        assert_eq!(id, id2);

        println!("{id_txt}");
    }

    #[test]
    fn test_source_de_ser() {
        let system: Source<String> = Source::System;
        let system_text = serde_json::to_string(&system).unwrap();
        let system2 = serde_json::from_str(&system_text).unwrap();
        assert_eq!(system, system2);

        let user = Source::User("a".to_string());
        let user_text = serde_json::to_string(&user).unwrap();
        let user2 = serde_json::from_str(&user_text).unwrap();
        assert_eq!(user, user2);
        println!("{user_text} {system_text}");

        {
            let user = Source::User(IUserID::from("b".to_string()));
            let user_text = serde_json::to_string(&user).unwrap();
            let user2 = serde_json::from_str(&user_text).unwrap();
            assert_eq!(user, user2);
            println!("{user_text} {system_text}");
        }

        #[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
        struct Test {
            source: Source<String>,
        }

        let t_user = Test { source: user };
        let txt_user = serde_json::to_string(&t_user).unwrap();
        assert_eq!(t_user, serde_json::from_str(&txt_user).unwrap());
        let t_sys = Test { source: system };
        let txt_sys = serde_json::to_string(&t_sys).unwrap();
        assert_eq!(t_sys, serde_json::from_str(&txt_sys).unwrap());
        println!("{txt_user} {txt_sys}");
        #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
        struct Foo {
            thing: Class,
        }
        println!(
            "{}",
            serde_json::to_string(&Foo {
                thing: Class::Genre
            })
            .unwrap()
        );
    }
    #[tokio::test]
    async fn test_source_surreal() {
        use surrealdb::{engine::local::Mem, Surreal};

        #[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
        struct Foo {
            source: Source<RecordId>,
        }
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();
        db.query("DEFINE TABLE OVERWRITE properties TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;")
            .query(
                "DEFINE FIELD OVERWRITE source ON properties TYPE 'system' | record<users> \
                 PERMISSIONS FULL;",
            )
            .await
            .unwrap();
        let foo_struct = Foo {
            source: Source::System,
        };
        let mut r: Vec<Foo> = db
            .insert("properties")
            .content(foo_struct.clone())
            .await
            .unwrap();
        assert_eq!(foo_struct, r.pop().unwrap());
    }
}
