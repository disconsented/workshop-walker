use surrealdb::{IndexedResults, Surreal, engine::local::Db};
use surrealdb_core::{
    sql::{Data, Expr, InsertStatement},
    val::TableName,
};
use surrealdb_types::{SurrealValue, Value};
use tracing::{debug, error};

use crate::{
    db::{AppID, IAppID, ITagID, model::InternalTag},
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

        // The `tags` table is SCHEMAFULL and requires an `app_id` that isn't
        // part of `InternalTag`. Without it the INSERTs below coerce-fail and
        // are *silently* dropped by `INSERT IGNORE`, so the tag rows never get
        // created. Derive it from the owning app.
        let app_id: i64 = AppID::try_from(app.clone())
            .map_err(|error| TagError::Internal {
                msg: error.to_string(),
            })?
            .into();

        let mut query = self
            .db
            .query("BEGIN TRANSACTION;")
            .query("UPDATE $id SET tags = $tag_ids;");

        for tag in tags {
            let mut value = tag.into_value();
            if let Value::Object(obj) = &mut value {
                obj.insert("app_id", app_id);
            }
            let mut stmt = InsertStatement::default();
            stmt.ignore = true;
            stmt.data = Data::SingleExpression(Expr::from_public_value(value));
            stmt.into = Some(Expr::Table(TableName::from("tags".to_string())));
            query = query.query(stmt);
        }

        let query = query
            .query("COMMIT;")
            .bind(("id", app))
            .bind(("tag_ids", tag_ids));

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
    use surrealdb::{
        Surreal,
        engine::local::{Db, Mem},
    };

    use crate::{
        db::{IAppID, ITagID, model::InternalTag},
        domain::tags::TagsPort,
    };

    const APP: i64 = 4;

    /// Minimal in-memory copy of the production `apps` + `tags` schema that the
    /// tag upsert touches. `tags` is SCHEMAFULL and requires `app_id`.
    const SCHEMA: &str = "
        DEFINE TABLE tags TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
        DEFINE FIELD app_id ON tags TYPE int PERMISSIONS FULL;
        DEFINE FIELD display_name ON tags TYPE string PERMISSIONS FULL;
        DEFINE FIELD id ON tags TYPE string PERMISSIONS FULL;
        DEFINE INDEX field_app_id_tag ON tags FIELDS app_id, display_name;

        DEFINE TABLE apps TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
        DEFINE FIELD id ON apps TYPE int PERMISSIONS FULL;
        DEFINE FIELD tags ON apps TYPE array<record<tags>> DEFAULT [] VALUE $value.distinct() \
                          PERMISSIONS FULL;
    ";

    async fn setup() -> Surreal<Db> {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();
        db.query(SCHEMA)
            .await
            .unwrap()
            .check()
            .expect("schema setup failed");
        db.query("CREATE apps:4;")
            .await
            .unwrap()
            .check()
            .expect("failed to create app");
        db
    }

    fn tag(id: &str, display_name: &str) -> InternalTag {
        InternalTag {
            id: ITagID::from(id.to_string()),
            display_name: display_name.to_string(),
        }
    }

    /// `apps:4.tags` as a sorted list of `"tags:<key>"` strings.
    async fn app_tag_ids(db: &Surreal<Db>) -> Vec<String> {
        let mut r = db
            .query("SELECT VALUE tags.map(|$v| $v.to_string()) FROM ONLY apps:4;")
            .await
            .unwrap();
        let mut ids: Vec<String> = r.take(0).unwrap();
        ids.sort();
        ids
    }

    /// Rows in the `tags` table as `(id_key, display_name, app_id)`, sorted.
    async fn tag_rows(db: &Surreal<Db>) -> Vec<(String, String, i64)> {
        let mut r = db
            .query("SELECT record::id(id) AS id, display_name, app_id FROM tags;")
            .await
            .unwrap();
        let rows: Vec<serde_json::Value> = r.take(0).unwrap_or_default();
        let mut out: Vec<(String, String, i64)> = rows
            .into_iter()
            .map(|v| {
                (
                    v["id"].as_str().unwrap().to_string(),
                    v["display_name"].as_str().unwrap().to_string(),
                    v["app_id"].as_i64().unwrap(),
                )
            })
            .collect();
        out.sort();
        out
    }

    #[tokio::test]
    async fn upsert_tags_creates_tag_rows_and_links_app() {
        let db = setup().await;
        let silo = super::TagsSilo::new(db.clone());

        silo.upsert_tags(
            IAppID::from(APP),
            vec![tag("mod", "Mod"), tag("scenario", "Scenario")],
        )
        .await
        .expect("upsert_tags should succeed");

        assert_eq!(app_tag_ids(&db).await, vec!["tags:mod", "tags:scenario"]);
        assert_eq!(
            tag_rows(&db).await,
            vec![
                ("mod".to_string(), "Mod".to_string(), APP),
                ("scenario".to_string(), "Scenario".to_string(), APP),
            ]
        );
    }

    #[tokio::test]
    async fn upsert_tags_replaces_the_apps_tag_set() {
        let db = setup().await;
        let silo = super::TagsSilo::new(db.clone());

        silo.upsert_tags(IAppID::from(APP), vec![tag("mod", "Mod")])
            .await
            .unwrap();
        // A second upsert overwrites the app's tag list with the new set.
        silo.upsert_tags(IAppID::from(APP), vec![tag("scenario", "Scenario")])
            .await
            .unwrap();

        assert_eq!(app_tag_ids(&db).await, vec!["tags:scenario"]);
    }

    #[tokio::test]
    async fn upsert_tags_is_idempotent_for_repeated_tags() {
        let db = setup().await;
        let silo = super::TagsSilo::new(db.clone());

        silo.upsert_tags(IAppID::from(APP), vec![tag("mod", "Mod")])
            .await
            .unwrap();
        // Re-inserting the same tag must not error (INSERT IGNORE) or duplicate.
        silo.upsert_tags(IAppID::from(APP), vec![tag("mod", "Mod")])
            .await
            .unwrap();

        assert_eq!(app_tag_ids(&db).await, vec!["tags:mod"]);
        assert_eq!(
            tag_rows(&db).await,
            vec![("mod".to_string(), "Mod".to_string(), APP)]
        );
    }
}
