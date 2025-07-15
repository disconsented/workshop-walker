use std::{
    fmt::{Display, Formatter},
    ops::{Deref, DerefMut},
};

use chrono::{DateTime, Utc};
use salvo::prelude::ToSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};
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
    pub properties: Vec<WorkshopItemProperties<String, Property>>,
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
    pub properties: Vec<WorkshopItemProperties<String, Property>>, // Approved or owned properties
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
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
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
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
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
pub fn serialize_chrono_as_sql_datetime<S>(
    x: &chrono::DateTime<Utc>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    Into::<surrealdb::sql::Datetime>::into(*x).serialize(s)
}

/// Crowdsourced metadata for an item, private version
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, Eq, PartialEq, Hash)]
pub struct Property {
    pub class: Class,
    pub value: String,
}
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
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
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
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
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
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
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct Vote {
    /// The app this is associated with, for possible filtering
    pub app_id: String,
    pub score: f32,
    pub when: DateTime<Utc>,
}

#[derive(Clone, Debug, ToSchema, Eq, PartialEq)]
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
    fn serialize<__S>(&self, __serializer: __S) -> Result<__S::Ok, __S::Error>
    where
        __S: Serializer,
    {
        match *self {
            Source::System => __serializer.serialize_str("system"),
            Source::User(ref __field0) => (*__field0).serialize(__serializer),
        }
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Source<T> {
    fn deserialize<D>(deserializer: D) -> Result<Source<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let __content =
            <serde::__private::de::Content as serde::Deserialize>::deserialize(deserializer)?;
        let __deserializer =
            serde::__private::de::ContentRefDeserializer::<D::Error>::new(&__content);

        if let Ok(__ok) = match String::deserialize(__deserializer) {
            Ok(str) if str == "system" => Ok(Source::System),

            Err(__err) => Err(__err),
            Ok(variant) => Err(D::Error::unknown_variant(&variant, &["system"])),
        } {
            return Ok(__ok);
        }
        if let Ok(__ok) = Result::map(
            <T as serde::Deserialize>::deserialize(__deserializer),
            Source::User,
        ) {
            return Ok(__ok);
        }

        Err(D::Error::custom(
            "Expected either T or 'system' got neither",
        ))
    }
}

#[derive(Debug, ToSchema, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
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
    use surrealdb::RecordId;

    use crate::model::{Class, Id, Source};

    #[test]
    fn test_id_newtype() {
        let id: Id = RecordId::from_table_key("items", "1").into();
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
            let user = Source::User(RecordId::from_table_key("a", "b"));
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
        use surrealdb::{Surreal, engine::local::Mem};

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
