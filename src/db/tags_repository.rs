use surrealdb::{engine::local::Db, IndexedResults, Surreal};
use tracing::{debug, error};

use crate::{
    db::{model::InternalTag, IAppID, ITagID},
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
    async fn upsert_tags(&self, app: IAppID, tags: Vec<InternalTag>) -> Result<(), TagError> {
        let tag_ids = tags
            .iter()
            .map(|tag| tag.id.clone())
            .collect::<Vec<ITagID>>();
        let query = self
            .db
            .query("BEGIN TRANSACTION;")
            .query("INSERT IGNORE INTO tags $tags;")
            .query("UPDATE $id SET tags = $tag_ids;")
            .query("COMMIT;")
            .bind(("id", app))
            .bind(("tag_ids", tag_ids))
            .bind(("tags", tags));


        debug!(?query, "upsert tags");

        if let Err(error) = query.await.map(IndexedResults::check) {
            error!(?error, "failed to upsert tag");
            return Err(TagError::Internal {
                msg: error.to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use surrealdb::{engine::local::Mem, Surreal};

    use crate::db::{IAppID, ITagID};

    #[tokio::test]
    async fn test_upsert_tags() {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();
        db.query(
            "DEFINE TABLE tags TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

            -- ------------------------------
            -- FIELDS
            -- ------------------------------ 
            
            DEFINE FIELD app ON tags TYPE int PERMISSIONS FULL;
            DEFINE FIELD display_name ON tags TYPE string PERMISSIONS FULL;
            DEFINE FIELD id ON tags TYPE string PERMISSIONS FULL;
            
            -- ------------------------------
            -- INDEXES
            -- ------------------------------ 
            
            DEFINE INDEX field_app_tag ON tags FIELDS app, display_name;
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
            .bind(("id", IAppID::from(4i64)))
            .await
            .unwrap();
        db.query("UPDATE $id SET tags = tags.add($record)")
            .bind(("id", IAppID::from(4i64)))
            .bind(("record", ITagID::from("something".to_string())))
            .await
            .unwrap();
        let stuff: Vec<String> = db
            .query("SELECT tags.map(|$v|$v.to_string()) FROM $id")
            .bind(("id", IAppID::from(4i64)))
            .await
            .unwrap()
            .take(0)
            .unwrap();
        assert_eq!(stuff, vec!["tags:something"]);
    }
}
