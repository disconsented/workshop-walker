use std::result::Result;

use surrealdb::{RecordId, Surreal, engine::local::Db};
use tracing::{debug, error};

use crate::{
    db::{
        ItemID, UserID,
        model::{Property, Source, WorkshopItemProperties},
    },
    domain::properties::{NewProperty, PropertiesError, PropertiesPort, VoteData},
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
        new_property: NewProperty,
        source: Source<String>,
    ) -> Result<(), PropertiesError> {
        let workshop_id = ItemID::from(new_property.workshop_item).into_recordid();

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
            debug!("Succeeded similar_properties test");
            let existing_properties: Vec<WorkshopItemProperties<String, Property>> =
                res.take(1).unwrap_or_default();
            existing_properties
                .iter()
                .any(|prop| prop.property == test_prop)
        };

        debug!(%test_prop, exists = prop_exists, "property already exists?");

        // Insert any new properties and relate
        match self
            .db
            .query(
                "INSERT IGNORE INTO properties (id) values (properties:{class: $class, value: \
                 $value});",
            )
            .bind(("class", test_prop.class))
            .bind(("value", test_prop.value))
            .query(
                "RELATE $workshop_id->workshop_item_properties->properties:{class: $class, \
                 value:$value} SET note=$note, source=$source;",
            )
            .bind(("workshop_id", workshop_id))
            .bind(("note", new_property.note))
            .bind((
                "source",
                match source {
                    Source::System => Source::System,
                    Source::User(userid) => Source::<RecordId>::User(UserID::from(userid).into()),
                },
            ))
            .await
            .map(surrealdb::Response::check)
        {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(surrealdb::Error::Db(surrealdb::err::Error::IndexExists { .. }))) => {
                Err(PropertiesError::Conflict)
            }
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

    async fn vote(&self, vote_data: VoteData, userid: String) -> Result<(), PropertiesError> {
        let user = UserID::from(userid);
        let query = self
            .db
            .query("LET $link = properties:{class: $class, value: $value}")
            .query(r#"IF !record::exists($link){THROW "FAIL LINK";}"#)
            .query(r#"IF !record::exists($item){THROW "FAIL ITEM";}"#)
            .query(
                "LET $changed = INSERT INTO votes (id, score, when) VALUES ({link: $link, user: \
                 $user, item: $item}, $score, time::now()) ON DUPLICATE KEY UPDATE when=time::now(), \
                 score=$score RETURN BEFORE;",
            )
            .query(
                r"
            LET $changed_score = $changed.score[0];
            IF $changed_score && $changed_score != $score{
                LET $vote_diff = ($changed_score * -1);
                UPDATE ONLY workshop_item_properties SET vote_count += $vote_count, upvote_count += $vote_diff WHERE in = $item AND out = $link;
                RETURN $vote_diff
            } else if !$changed_score{
                UPDATE ONLY workshop_item_properties SET vote_count += 1, upvote_count += $score WHERE in = $item AND out = $link;
            } else {
                return {changed_score: $changed_score, score: $score};
            };",
            )
            .bind(("class", vote_data.class))
            .bind(("value", vote_data.value))
            .bind(("user", user.into_recordid()))
            .bind((
                "item",
                RecordId::from_table_key("workshop_items", vote_data.item),
            ))
            .bind(("score", vote_data.score));

        match query.await.map(surrealdb::Response::check) {
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
        vote_data: VoteData,
        userid: String,
    ) -> Result<(), PropertiesError> {
        let user = UserID::from(userid);
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
            .bind(("user", user.into_recordid()))
            .bind((
                "item",
                RecordId::from_table_key("workshop_items", vote_data.item),
            ))
            .await;

        match result.map(surrealdb::Response::check) {
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
