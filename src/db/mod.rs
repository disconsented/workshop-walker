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

define_id!("users", IUserID, UserID, String);
define_id!("workshop_items", ItemID, IItemID, String);





#[cfg(test)]
mod test {
    use surrealdb::{Surreal, engine::local::Mem};

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
