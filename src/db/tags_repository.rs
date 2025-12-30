use surrealdb::{RecordId, Surreal, engine::local::Db};
use tracing::error;

use crate::{
    db::model::Tag,
    domain::tags::{TagError, TagsPort},
};

pub struct TagsSilo {
    db: Surreal<Db>,
}

impl TagsSilo {
    pub fn new(db: Surreal<Db>) -> Self {
        Self { db }
    }
}

impl TagsPort for TagsSilo {
    async fn upsert_tags(&self, app_id: u32, tags: Vec<Tag>) -> Result<(), TagError> {
        for tag in tags {
            let query = self
                .db
                .query("BEGIN TRANSACTION")
                .query("UPSERT tags CONTENT $tag")
                .query("UPDATE $id SET tags = tags.add($record)")
                .query("COMMIT")
                .bind(("record", RecordId::from_table_key("tags", &tag.tag)))
                .bind(("tag", tag))
                .bind(("id", RecordId::from_table_key("apps", i64::from(app_id))));
            if let Err(error) = query.await.map(surrealdb::Response::check) {
                error!(?error, "failed to upsert tag");
                return Err(TagError::Internal {
                    msg: error.to_string(),
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use surrealdb::{RecordId, Surreal, engine::local::Mem};

    #[tokio::test]
    async fn test_upsert_tags() {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();
        db.query(
            "DEFINE TABLE tags TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

            -- ------------------------------
            -- FIELDS
            -- ------------------------------ 
            
            DEFINE FIELD app_id ON tags TYPE int PERMISSIONS FULL;
            DEFINE FIELD display_name ON tags TYPE string PERMISSIONS FULL;
            DEFINE FIELD id ON tags TYPE string PERMISSIONS FULL;
            
            -- ------------------------------
            -- INDEXES
            -- ------------------------------ 
            
            DEFINE INDEX field_app_id_tag ON tags FIELDS app_id, display_name;
            DEFINE TABLE apps TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

            -- ------------------------------
            -- FIELDS
            -- ------------------------------


            DEFINE FIELD id ON apps TYPE int PERMISSIONS FULL;
            DEFINE FIELD tags ON apps TYPE set<record<tags>> DEFAULT [] PERMISSIONS FULL;
            DEFINE FIELD tags[*] ON apps TYPE record<tags> PERMISSIONS FULL;

            -- ------------------------------
            -- INDEXES
            -- ------------------------------

            DEFINE INDEX apps_id ON apps FIELDS id;",
        )
        .await
        .unwrap();
        db.query("CREATE $id")
            .bind(("id", RecordId::from_table_key("apps", 4)))
            .await
            .unwrap();
        db.query("UPDATE $id SET tags = tags.add($record)")
            .bind(("id", RecordId::from_table_key("apps", 4)))
            .bind(("record", RecordId::from_table_key("tags", "something")))
            .await
            .unwrap();
        let stuff: Vec<String> = db
            .query("SELECT tags.map(|$v|$v.to_string()) FROM $id")
            .bind(("id", RecordId::from_table_key("apps", 4)))
            .await
            .unwrap()
            .take(0)
            .unwrap();
        assert_eq!(stuff, vec!["tags:something"]);
    }
}
