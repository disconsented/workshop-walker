mod app;
mod class;
mod full_workshop_item;
mod insertable_workshop_item;
mod order_by;
mod property;
mod source;
mod status;
mod tag;
mod user;
mod username;
mod vote;
mod workshop_item;
mod workshop_item_properties;

pub use app::*;
pub use class::*;
pub use full_workshop_item::*;
pub use insertable_workshop_item::*;
pub use order_by::*;
pub use property::*;
pub use source::*;
pub use status::*;
use surrealdb_types::SurrealValue;
pub use tag::*;
pub use user::*;
pub use username::*;
pub use workshop_item::*;
pub use workshop_item_properties::*;

// I'd love for this to be a VecDeque but surreal breaks and deserialize an
// array to it. Silly little struct for yanking history out of the DB for
// updating.
#[derive(SurrealValue, Default)]
pub struct HistoryPair {
    pub view_history: Vec<u64>,
    pub subscription_history: Vec<u64>,
}

#[cfg(test)]
mod test {
    use surrealdb_types::{RecordIdKey, SurrealValue, Value};

    use crate::db::model::{class::Class, property::Property};

    /// Pins the root cause of `properties:{class: {Type: {}}, value: 'asd'}`.
    ///
    /// A `class` is a unit-like enum variant; when bound into a record-id key
    /// it must serialise to the bare string `"Type"`. The current
    /// `into_value` impl emits the externally-tagged object `{"Type": {}}`
    /// instead, which is exactly what leaks into the stored record id.
    #[test]
    fn class_serialises_to_bare_string() {
        let value = Class::Type.into_value();
        assert_eq!(
            value,
            Value::String("Type".to_string()),
            "Class should serialise to a bare string for record-id keys, but produced the \
             externally-tagged object form: {value:?}"
        );
    }

    /// Reproduces the symptom end-to-end: building the record-id key for a
    /// property yields `{class: {Type: {}}, value: 'asd'}` instead of
    /// `{class: 'Type', value: 'asd'}`.
    #[test]
    fn property_record_key_uses_bare_class_string() {
        let prop = Property {
            class: Class::Type,
            value: "asd".to_string(),
        };

        let key: RecordIdKey = prop.into();
        let RecordIdKey::Object(obj) = key else {
            panic!("expected an object record-id key, got: {key:?}");
        };

        assert_eq!(
            obj.get("class"),
            Some(&Value::String("Type".to_string())),
            "the `class` part of the record-id key must be the bare string 'Type', but was: {:?}",
            obj.get("class")
        );
    }

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
