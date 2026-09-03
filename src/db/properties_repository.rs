use std::result::Result;

use surrealdb::{Surreal, engine::local::Db};
use tracing::{debug, error};

use crate::{
    db::{
        IUserID,
        model::{InternalSource, InternalWorkshopItemProperties, Property, Status},
    },
    domain::properties::{InternalNewProperty, InternalVoteData, PropertiesError, PropertiesPort},
};

pub struct PropertiesSilo {
    pub db: Surreal<Db>,
}

impl PropertiesSilo {
    pub fn new(db: Surreal<Db>) -> Self {
        Self { db }
    }
}

impl PropertiesPort for PropertiesSilo {
    async fn create_or_link_property(
        &self,
        new_property: InternalNewProperty,
        source: InternalSource,
        status: Status,
    ) -> Result<(), PropertiesError> {
        let workshop_id = new_property.workshop_item;

        let test_prop = Property {
            class: new_property.class,
            value: new_property.value,
        };

        // Similarity and existence checks
        let prop_exists = {
            let query = self
                .db
                .query(
                    "SELECT id.class as class, id.value as value FROM properties WHERE \
                     string::distance::damerau_levenshtein(id.value, $value) < 0.8;",
                )
                .query(
                    "SELECT *, in.to_string(), out.*.id.{class,value} as out, source.to_string() \
                     OMIT id FROM workshop_item_properties WHERE in=$workshop_item",
                )
                .bind(("workshop_item", workshop_id.clone()))
                .bind(("value", test_prop.value.clone()));
            let res = match query.await {
                Ok(r) => r,
                Err(e) => {
                    error!(?e, "query failed");
                    return Err(PropertiesError::Internal);
                }
            };
            let mut res = match res.check() {
                Ok(r) => r,
                Err(e) => {
                    error!(?e, "check failed");
                    return Err(PropertiesError::Internal);
                }
            };

            let similar_properties = res.take::<Vec<Property>>(0).unwrap_or_default();
            if !similar_properties.is_empty() && !similar_properties.contains(&test_prop) {
                debug!(?similar_properties, "Similar properties exist");
                return Err(PropertiesError::Conflict);
            }
            let existing_properties: Vec<InternalWorkshopItemProperties> =
                res.take(1).unwrap_or_default();
            existing_properties.iter().any(|prop| prop.out == test_prop)
        };

        debug!(%test_prop, exists = prop_exists, "property already exists?");

        // Insert any new properties and relate
        match self
            .db
            .query("BEGIN")
            .query(
                "INSERT IGNORE INTO properties (id) values (properties:{class: $class, value: \
                 $value});",
            )
            .bind(("class", test_prop.class))
            .bind(("value", test_prop.value))
            .query(
                "RELATE $workshop_id->workshop_item_properties->properties:{class: $class, \
                 value:$value} SET note=$note, source=$source, status=$status;",
            )
            .bind(("workshop_id", workshop_id))
            .bind(("note", new_property.note))
            .bind((
                "source",
                match source {
                    InternalSource::System => InternalSource::System,
                    InternalSource::User(userid) => InternalSource::User(userid),
                },
            ))
            .bind(("status", status))
            .query("COMMIT")
            .await
            .map(surrealdb::IndexedResults::check)
        {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(err)) if err.is_already_exists() => Err(PropertiesError::Conflict),
            Ok(Err(other)) => {
                error!(?other, "unexpected DB error");
                Err(PropertiesError::Internal)
            }
            Err(e) => {
                error!(?e, "query error");
                Err(PropertiesError::Internal)
            }
        }
    }

    async fn vote(
        &self,
        vote_data: InternalVoteData,
        user: IUserID,
    ) -> Result<(), PropertiesError> {
        let query = self
            .db
            .query("BEGIN")
            .query("LET $link = properties:{class: $class, value: $value}")
            .query(r#"IF !record::exists($link){THROW "FAIL LINK";}"#)
            .query(r#"IF !record::exists($item){THROW "FAIL ITEM";}"#)
            .query(
                "LET $changed = INSERT INTO votes (id, score, when, user) VALUES ({link: $link, \
                 user: $user, item: $item}, $score, time::now(), $user) ON DUPLICATE KEY UPDATE \
                 when=time::now(), score=$score RETURN BEFORE;",
            )
            .query(
                r"
            LET $changed_score = $changed[0].score;
            IF $changed_score && $changed_score != $score{
                -- Existing vote whose score flipped: total vote count is unchanged,
                -- only the net (upvote) tally shifts by the delta.
                UPDATE ONLY workshop_item_properties SET upvote_count += ($score - $changed_score) WHERE in = $item AND out = $link;
            } else if !$changed_score{
                -- Brand new vote for this (item, property, user).
                UPDATE ONLY workshop_item_properties SET vote_count += 1, upvote_count += $score WHERE in = $item AND out = $link;
            };",
            )
            .query("COMMIT")
            .bind(("class", vote_data.class))
            .bind(("value", vote_data.value))
            .bind(("user", user))
            .bind((
                "item",
                vote_data.item,
            ))
            .bind(("score", vote_data.score));

        match query.await.map(surrealdb::IndexedResults::check) {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => {
                debug!(?e, "bad vote from user");
                Err(PropertiesError::BadRequest {
                    msg: "Invalid vote".into(),
                })
            }
            Err(e) => {
                error!(?e, "vote query error");
                Err(PropertiesError::Internal)
            }
        }
    }

    async fn remove_vote(
        &self,
        vote_data: InternalVoteData,
        user: IUserID,
    ) -> Result<(), PropertiesError> {
        let result = self
            .db
            .query("BEGIN TRANSACTION;")
            .query("LET $link = properties:{class: $class, value: $value}")
            .query(
                "let $before = DELETE only votes:{link: $link, user: $user, item: $item} RETURN \
                 BEFORE;",
            )
            .query(
                "if $before.score{RETURN UPDATE ONLY workshop_item_properties SET \
                 vote_count=math::max([vote_count-1, 0]), upvote_count-=$before.score WHERE \
                 in=$item AND out=$link RETURN diff};",
            )
            .query("COMMIT TRANSACTION;")
            .bind(("class", vote_data.class))
            .bind(("value", vote_data.value))
            .bind(("user", user))
            .bind(("item", vote_data.item))
            .await;

        match result.map(surrealdb::IndexedResults::check) {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => {
                debug!(?e, "bad vote removal from user");
                Err(PropertiesError::BadRequest {
                    msg: "Invalid removal".into(),
                })
            }
            Err(e) => {
                error!(?e, "vote removal query error");
                Err(PropertiesError::Internal)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use surrealdb::{
        Surreal,
        engine::local::{Db, Mem},
    };

    use super::PropertiesSilo;
    use crate::{
        db::{IItemID, IUserID, model::Class},
        domain::properties::{InternalVoteData, PropertiesError, PropertiesPort},
    };

    const ITEM: i64 = 294100;

    /// Minimal in-memory copy of the production (SCHEMAFULL) schema the vote
    /// flow touches.
    ///
    /// On the `workshop_item_properties` relation:
    ///   * `vote_count`   = total number of votes cast (up + down)
    ///   * `upvote_count` = net score (sum of every voter's +1 / -1)
    const SCHEMA: &str = "
        DEFINE TABLE workshop_items TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
        DEFINE FIELD name ON workshop_items TYPE option<string> PERMISSIONS FULL;

        DEFINE TABLE properties TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
        DEFINE FIELD id ON properties TYPE { class: string, value: string } PERMISSIONS FULL;

        DEFINE TABLE users TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

        DEFINE TABLE workshop_item_properties TYPE RELATION IN workshop_items OUT properties \
                          SCHEMAFULL PERMISSIONS NONE;
        DEFINE FIELD note ON workshop_item_properties TYPE none | string PERMISSIONS FULL;
        DEFINE FIELD source ON workshop_item_properties TYPE 'system' | record<users> PERMISSIONS \
                          FULL;
        DEFINE FIELD status ON workshop_item_properties TYPE -1 | 0 | 1 DEFAULT 0 PERMISSIONS FULL;
        DEFINE FIELD upvote_count ON workshop_item_properties TYPE int DEFAULT 0 PERMISSIONS FULL;
        DEFINE FIELD vote_count ON workshop_item_properties TYPE int DEFAULT 0 PERMISSIONS FULL;
        DEFINE INDEX unique_workshop_item_properties ON workshop_item_properties FIELDS in, out \
                          UNIQUE;

        DEFINE TABLE votes TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;
        DEFINE FIELD id ON votes TYPE { item: record<workshop_items>, link: record<properties>, \
                          user: record<users> } PERMISSIONS FULL;
        DEFINE FIELD score ON votes TYPE int PERMISSIONS FULL;
        DEFINE FIELD user ON votes TYPE record<users> PERMISSIONS FULL;
        DEFINE FIELD when ON votes TYPE datetime PERMISSIONS FULL;
    ";

    /// Build a `PropertiesSilo` over an in-memory DB seeded with one item, one
    /// `Feature/ffff` property, two users, and the link between item and
    /// property. Returns the silo plus a handle to the same DB for assertions.
    async fn setup() -> (PropertiesSilo, Surreal<Db>) {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();

        db.query(SCHEMA)
            .await
            .unwrap()
            .check()
            .expect("schema setup failed");

        db.query("CREATE workshop_items:294100 SET name = 'test';")
            .query(
                "INSERT INTO properties (id) VALUES (properties:{class:'Feature', value:'ffff'});",
            )
            .query("CREATE users:1; CREATE users:2;")
            .query(
                "RELATE workshop_items:294100 -> workshop_item_properties -> \
                 properties:{class:'Feature', value:'ffff'} SET note = NONE, source = 'system', \
                 status = 0;",
            )
            .await
            .unwrap()
            .check()
            .expect("seed data failed");

        (PropertiesSilo::new(db.clone()), db)
    }

    /// A vote for the seeded `Feature/ffff` property on the seeded item.
    fn vote(score: i32) -> InternalVoteData {
        InternalVoteData {
            item: IItemID::from(ITEM),
            class: Class::Feature,
            value: "ffff".to_string(),
            score,
        }
    }

    fn user(id: i64) -> IUserID {
        IUserID::from(id)
    }

    /// `(vote_count, upvote_count)` on the seeded relation.
    async fn counts(db: &Surreal<Db>) -> (i64, i64) {
        let mut r = db
            .query("SELECT vote_count, upvote_count FROM ONLY workshop_item_properties LIMIT 1;")
            .await
            .unwrap();
        let v: serde_json::Value = r
            .take::<Option<serde_json::Value>>(0)
            .unwrap()
            .expect("the seeded relation should always exist");
        (
            v["vote_count"].as_i64().unwrap(),
            v["upvote_count"].as_i64().unwrap(),
        )
    }

    /// Number of rows currently in the `votes` table.
    async fn vote_rows(db: &Surreal<Db>) -> usize {
        let mut r = db.query("SELECT VALUE id FROM votes;").await.unwrap();
        r.take::<Vec<serde_json::Value>>(0)
            .unwrap_or_default()
            .len()
    }

    // --- adding a vote to a property (none -> vote) ---

    #[tokio::test]
    async fn add_upvote_sets_score() {
        let (silo, db) = setup().await;
        silo.vote(vote(1), user(1)).await.unwrap();
        assert_eq!(counts(&db).await, (1, 1));
        assert_eq!(vote_rows(&db).await, 1);
    }

    #[tokio::test]
    async fn add_downvote_sets_score() {
        let (silo, db) = setup().await;
        silo.vote(vote(-1), user(1)).await.unwrap();
        assert_eq!(counts(&db).await, (1, -1));
        assert_eq!(vote_rows(&db).await, 1);
    }

    // --- changing a vote (vote -> different vote): score must follow ---

    #[tokio::test]
    async fn change_upvote_to_downvote_updates_score() {
        let (silo, db) = setup().await;
        silo.vote(vote(1), user(1)).await.unwrap();
        assert_eq!(counts(&db).await, (1, 1));
        silo.vote(vote(-1), user(1)).await.unwrap();
        // total votes unchanged, net score swings +1 -> -1
        assert_eq!(counts(&db).await, (1, -1));
        assert_eq!(vote_rows(&db).await, 1);
    }

    #[tokio::test]
    async fn change_downvote_to_upvote_updates_score() {
        let (silo, db) = setup().await;
        silo.vote(vote(-1), user(1)).await.unwrap();
        assert_eq!(counts(&db).await, (1, -1));
        silo.vote(vote(1), user(1)).await.unwrap();
        assert_eq!(counts(&db).await, (1, 1));
        assert_eq!(vote_rows(&db).await, 1);
    }

    // --- re-casting the same vote is a no-op ---

    #[tokio::test]
    async fn recasting_same_upvote_is_noop() {
        let (silo, db) = setup().await;
        silo.vote(vote(1), user(1)).await.unwrap();
        silo.vote(vote(1), user(1)).await.unwrap();
        assert_eq!(counts(&db).await, (1, 1));
        assert_eq!(vote_rows(&db).await, 1);
    }

    #[tokio::test]
    async fn recasting_same_downvote_is_noop() {
        let (silo, db) = setup().await;
        silo.vote(vote(-1), user(1)).await.unwrap();
        silo.vote(vote(-1), user(1)).await.unwrap();
        assert_eq!(counts(&db).await, (1, -1));
        assert_eq!(vote_rows(&db).await, 1);
    }

    // --- removing a vote (vote -> none) ---

    #[tokio::test]
    async fn remove_upvote_clears_score() {
        let (silo, db) = setup().await;
        silo.vote(vote(1), user(1)).await.unwrap();
        silo.remove_vote(vote(1), user(1)).await.unwrap();
        assert_eq!(counts(&db).await, (0, 0));
        assert_eq!(vote_rows(&db).await, 0);
    }

    #[tokio::test]
    async fn remove_downvote_clears_score() {
        let (silo, db) = setup().await;
        silo.vote(vote(-1), user(1)).await.unwrap();
        silo.remove_vote(vote(-1), user(1)).await.unwrap();
        assert_eq!(counts(&db).await, (0, 0));
        assert_eq!(vote_rows(&db).await, 0);
    }

    #[tokio::test]
    async fn remove_without_existing_vote_is_noop() {
        let (silo, db) = setup().await;
        silo.remove_vote(vote(1), user(1)).await.unwrap();
        assert_eq!(counts(&db).await, (0, 0));
        assert_eq!(vote_rows(&db).await, 0);
    }

    // --- multiple voters: the score aggregates every voter ---

    #[tokio::test]
    async fn two_users_upvoting_sum_the_score() {
        let (silo, db) = setup().await;
        silo.vote(vote(1), user(1)).await.unwrap();
        silo.vote(vote(1), user(2)).await.unwrap();
        assert_eq!(counts(&db).await, (2, 2));
        assert_eq!(vote_rows(&db).await, 2);
    }

    #[tokio::test]
    async fn opposing_votes_net_to_zero() {
        let (silo, db) = setup().await;
        silo.vote(vote(1), user(1)).await.unwrap();
        silo.vote(vote(-1), user(2)).await.unwrap();
        assert_eq!(counts(&db).await, (2, 0));
        assert_eq!(vote_rows(&db).await, 2);
    }

    #[tokio::test]
    async fn one_user_changing_does_not_affect_the_other() {
        let (silo, db) = setup().await;
        silo.vote(vote(1), user(1)).await.unwrap();
        silo.vote(vote(1), user(2)).await.unwrap();
        // user 2 flips to a downvote: still two votes, net back to zero
        silo.vote(vote(-1), user(2)).await.unwrap();
        assert_eq!(counts(&db).await, (2, 0));
        assert_eq!(vote_rows(&db).await, 2);
    }

    #[tokio::test]
    async fn removing_one_of_two_votes_updates_score() {
        let (silo, db) = setup().await;
        silo.vote(vote(1), user(1)).await.unwrap();
        silo.vote(vote(1), user(2)).await.unwrap();
        silo.remove_vote(vote(1), user(1)).await.unwrap();
        assert_eq!(counts(&db).await, (1, 1));
        assert_eq!(vote_rows(&db).await, 1);
    }

    // --- voting against records that don't exist is rejected ---

    #[tokio::test]
    async fn vote_on_missing_property_errors() {
        let (silo, db) = setup().await;
        let missing = InternalVoteData {
            item: IItemID::from(ITEM),
            class: Class::Feature,
            value: "does-not-exist".to_string(),
            score: 1,
        };
        let err = silo.vote(missing, user(1)).await.unwrap_err();
        assert!(matches!(err, PropertiesError::BadRequest { .. }));
        assert_eq!(counts(&db).await, (0, 0));
        assert_eq!(vote_rows(&db).await, 0);
    }

    #[tokio::test]
    async fn vote_on_missing_item_errors() {
        let (silo, db) = setup().await;
        let missing = InternalVoteData {
            item: IItemID::from(999_i64),
            class: Class::Feature,
            value: "ffff".to_string(),
            score: 1,
        };
        let err = silo.vote(missing, user(1)).await.unwrap_err();
        assert!(matches!(err, PropertiesError::BadRequest { .. }));
        assert_eq!(vote_rows(&db).await, 0);
    }
}
