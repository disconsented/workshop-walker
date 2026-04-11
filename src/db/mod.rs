pub mod admin_actor;
pub mod admin_repository;
pub mod apps_actor;
pub mod apps_repository;
pub mod item_update_actor;
pub mod model;
pub mod properties_actor;
pub mod properties_repository;
pub mod tags_repository;
pub mod user_names_repository;

use macros::define_id;
use surrealdb::{engine::local::Mem, Surreal};
// use salvo::prelude::ToSchema;
use surrealdb_types::SurrealValue;
use crate::db::model::InternalTag;

define_id!("users", IUserID, UserID, String);

// i64 is due to `impl From<i64> for RecordIdKey` rather than `impl From<u64>
// for RecordId`
define_id!("workshop_items", IItemID, ItemID, i64);
define_id!("apps", IAppID, AppID, i64);
define_id!("tags", ITagID, TagID, InternalTag);
define_id!("usernames", IUsernameID, UsernameID, i64);
define_id!("properties", IPropertyID, PropertyID, String);
define_id!("votes", IVoteID, VoteID, String);
define_id!(
    "item_dependencies",
    IItemDependencyID,
    ItemDependencyID,
    Vec<surrealdb_types::Value>
);
define_id!("companions", ICompanionID, CompanionID, String);
define_id!(
    "workshop_item_properties",
    IWorkshopItemPropertyID,
    WorkshopItemPropertyID,
    Vec<surrealdb_types::Value>
);

#[cfg(test)]
mod test {
    use surrealdb::{engine::local::Mem, Surreal};

    use crate::db::*;

    #[test]
    fn test_type_conversion() {
        use std::convert::TryInto;

        use surrealdb_types::RecordId;

        fn test_id<I, E, T>(table: &str, key: T)
        where
            I: From<E> + TryInto<E> + Into<RecordId> + Clone,
            E: From<T> + Into<T> + Clone + std::fmt::Debug + PartialEq,
            T: Clone + PartialEq + std::fmt::Debug,
            <I as TryInto<E>>::Error: std::fmt::Debug,
        {
            let external = E::from(key.clone());
            let internal: I = I::from(external.clone());
            let record_id: RecordId = internal.clone().into();

            assert_eq!(record_id.table(), table);
            // key equality check depends on how it's stored, but we can check conversion
            // back
            let external_back: E = internal.try_into().unwrap();
            assert_eq!(external, external_back);
        }

        test_id::<IUserID, UserID, String>("users", "alice".to_string());
        test_id::<IItemID, ItemID, String>("workshop_items", "item1".to_string());
        test_id::<IAppID, AppID, i64>("apps", 123);
        test_id::<ITagID, TagID, String>("tags", "rpg".to_string());
        test_id::<IUsernameID, UsernameID, i64>("usernames", 456);
        test_id::<IPropertyID, PropertyID, String>("properties", "prop1".to_string());
        test_id::<IVoteID, VoteID, String>("votes", "vote1".to_string());
        test_id::<ICompanionID, CompanionID, String>("companions", "comp1".to_string());

        // For complex IDs, manual check
        let key = vec![
            surrealdb_types::Value::from("a"),
            surrealdb_types::Value::from("b"),
        ];
        let external = ItemDependencyID::from(key.clone());
        let internal = IItemDependencyID::from(external.clone());
        let external_back: ItemDependencyID = internal.try_into().unwrap();
        let key_back: Vec<surrealdb_types::Value> = external_back.into();
        assert_eq!(key, key_back);

        let external_prop = WorkshopItemPropertyID::from(key.clone());
        let internal_prop = IWorkshopItemPropertyID::from(external_prop.clone());
        let external_prop_back: WorkshopItemPropertyID = internal_prop.try_into().unwrap();
        let key_prop_back: Vec<surrealdb_types::Value> = external_prop_back.into();
        assert_eq!(key, key_prop_back);
    }

    #[tokio::test]
    async fn test_throw() {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();
        let output = db
            .query("DEFINE TABLE OVERWRITE properties TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;")
            .query("CREATE properties:ShouldExist;")
            .query("BEGIN TRANSACTION;")
            .query("CREATE properties:NotExist;")
            .query("IF true{THROW \"GRACEFUL\"};")
            .query("SELECT * FROM properties;")
            .await
            .unwrap();
        dbg!(output.check().unwrap());
    }

    #[tokio::test]
    async fn test_throw_commit() {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();
        let query = r#"BEGIN;
            LET $person = CREATE ONLY person SET age = rand::int(1,5);
            IF $person.age == 5 {
                THROW "Whoops";
            };
            CREATE person;
            COMMIT;"#;
        let output = db.query(query).await.unwrap();
        dbg!(output.check().unwrap());
    }
}
