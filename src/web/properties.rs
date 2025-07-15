use biscuit_auth::Authorizer;
use chrono::{DateTime, Utc};
use log::{debug, error};
use salvo::{
    Depot, Response, Writer,
    oapi::extract::JsonBody,
    prelude::{StatusCode, ToSchema, endpoint},
};
use serde::{Deserialize, Serialize};
use surrealdb::{RecordId, Surreal, engine::local::Db};

use crate::{
    db::{ItemID, UserID},
    model::{Class, Property, Source, WorkshopItemProperties},
    web::DB_POOL,
};

/// Add or change a vote for a property.
/// Prop must be accepted otherwise, fail.
/// Must be either an upvote (1) or a downvote (0), not used for removal.
#[endpoint]
pub async fn vote(vote_data: JsonBody<VoteData>, depot: &mut Depot, response: &mut Response) {
    if vote_data.score != 1 && vote_data.score != -1 {
        response.status_code(StatusCode::BAD_REQUEST);
        return;
    }
    let db: &Surreal<Db> = DB_POOL.get().expect("DB not initialised");
    let user = crate::auth::get_user(depot)
        .await
        .expect("already authenticated");
    let user = UserID::from(user);
    let query = db
        // .query("BEGIN TRANSACTION;")
        .query("LET $link = properties:{class: $class, value: $value}")
        .query(r#"IF !record::exists($link){THROW "FAIL LINK";}"#)
        .query(r#"IF !record::exists($item){THROW "FAIL ITEM";}"#)
        .query(
            "LET $changed = INSERT INTO votes (id, score, when) VALUES ({link: $link, user: \
             $user, item: $item}, $score, time::now()) ON DUPLICATE KEY UPDATE when=time::now(), \
             score=$score RETURN BEFORE;",
        )
        // only increment the vote count on an insertion, as score is truthy
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
                return {chnaged_score: $changed_score, score: $score};
            };",
        )
        // .query("COMMIT TRANSACTION;")
        .bind(("class", vote_data.0.class))
        .bind(("value", vote_data.0.value))
        .bind(("user", user.into_recordid()))
        .bind((
            "item",
            RecordId::from_table_key("workshop_items", vote_data.0.item),
        ))
        .bind(("score", vote_data.0.score));
    let result = query.await;
    match result {
        Ok(e) => match e.check() {
            Ok(_) => {
                response.status_code(StatusCode::NO_CONTENT);
            }
            Err(err) => {
                debug!("bad vote from user: {err}");
                response.status_code(StatusCode::BAD_REQUEST);
            }
        },
        Err(e) => {
            error!("vote query error: {e:?}");
            response.status_code(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}
#[endpoint]
pub async fn remove(vote_data: JsonBody<VoteData>, depot: &mut Depot, response: &mut Response) {
    let db: &Surreal<Db> = DB_POOL.get().expect("DB not initialised");
    let user = crate::auth::get_user(depot)
        .await
        .expect("already authenticated");
    let user = UserID::from(user);
    let result = db
        .query("BEGIN TRANSACTION;")
        .query("LET $link = properties:{class: $class, value: $value}")
        .query(
            "let $before = DELETE only votes:{link: $link, user: $user, item: $item} RETURN \
             BEFORE;",
        )
        .query(
            // Update if there _was_ an entry deleted
            "if $before.score{RETURN UPDATE ONLY workshop_item_properties SET \
             vote_count=math::max([vote_count-1, 0]), upvote_count-=$before.score WHERE in=$item \
             AND out=$link RETURN diff};",
        )
        .query("COMMIT TRANSACTION;")
        .bind(("class", vote_data.0.class))
        .bind(("value", vote_data.0.value))
        .bind(("user", user.into_recordid()))
        .bind((
            "item",
            RecordId::from_table_key("workshop_items", vote_data.0.item),
        ))
        .await;
    match result {
        Ok(e) => match e.check() {
            Ok(_) => {
                response.status_code(StatusCode::NO_CONTENT);
            }
            Err(err) => {
                debug!("bad vote from user: {err:?}");
                response.status_code(StatusCode::BAD_REQUEST);
            }
        },
        Err(e) => {
            error!("vote query error: {e:?}");
            response.status_code(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}
/// Add a new property with the following rules:
/// Must be entirely new or an exact match to an existing property.
/// Likeness checks are done on the value only (for now) using
/// damerau_levenshtein distance.
#[endpoint]
pub async fn new(new_property: JsonBody<NewProperty>, depot: &mut Depot, response: &mut Response) {
    let workshop_id = ItemID::from(new_property.0.workshop_item).into_recordid();
    // Select similar properties
    let db: &Surreal<Db> = DB_POOL.get().expect("DB not initialised");
    let test_prop = Property {
        class: new_property.0.class,
        value: new_property.0.value,
    };
    let prop_exists = {
        #[derive(Serialize, Deserialize, Clone, Debug)]
        struct Temp {
            id: surrealdb::Value,
        }
        let query = db
            .query(
                "SELECT id.class as class, id.value as value FROM properties WHERE \
                 string::distance::damerau_levenshtein(id.value, $value) < 0.8;",
            )
            .query(
                "SELECT *, in.to_string(), out.*.id.{class,value} as out, source.to_string() OMIT \
                 id FROM workshop_item_properties WHERE in=$workshop_item",
            )
            .bind(("workshop_item", workshop_id.clone()))
            .bind(("value", test_prop.value.clone()));
        let mut res = query.await.unwrap();
        res = res.check().unwrap();

        let similar_properties = res.take::<Vec<Property>>(0).unwrap();
        if !similar_properties.is_empty() && !similar_properties.contains(&test_prop) {
            // return similar entries
            error!("Similar props exist: {similar_properties:?}");
            response.status_code(StatusCode::CONFLICT);
            response.body(format!("{similar_properties:?}"));
            return;
        }
        debug!("Succeeded similar_properties test");
        let existing_properties: Vec<WorkshopItemProperties<String, Property>> =
            res.take(1).unwrap();
        existing_properties
            .iter()
            .any(|prop| prop.property == test_prop)
    };
    debug!("{test_prop:?} already exists? {prop_exists}");
    let authorizer = depot
        .obtain_mut::<Authorizer>()
        .expect("getting shared state");
    let (userid, _): (String, i64) = authorizer
        .query_exactly_one("data($user, 0) <- user($user)")
        .unwrap();

    // Insert any new properties

    {
        match db
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
            .bind(("note", new_property.0.note))
            .bind((
                "source",
                Source::<RecordId>::User(UserID::from(userid).into()),
            ))
            .await
            .unwrap()
            .check()
        {
            Ok(_) => {
                response.status_code(StatusCode::NO_CONTENT);
            }
            Err(surrealdb::Error::Db(surrealdb::err::Error::IndexExists { .. })) => {
                // Already exists, may be pending or rejected
                response.status_code(StatusCode::CONFLICT);
                response.body("Property already exists; Maybe pending or denied");
            }
            Err(other) => {
                error!("{other:?}");
                response.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }
}

/// Crowdsourced metadata for an item, private version
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct NewProperty {
    pub class: Class,
    pub value: String,
    /// Reasoning or justification for an inclusion
    pub note: Option<String>,
    pub workshop_item: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct VoteData {
    pub item: String,
    pub class: Class,
    pub value: String,
    // Must be -1 or 1 for now
    pub score: i8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdateVote {
    pub user: UserID,
    pub when: DateTime<Utc>,
}

#[cfg(test)]
mod test {

    use surrealdb::{Surreal, engine::local::Mem};

    use crate::model::{Class, Property};

    #[test]
    fn test_biscuit_conversion() {
        use biscuit_auth::{Biscuit, KeyPair};
        let keypair = KeyPair::new();
        let builder = Biscuit::builder().fact("user(\"John Doe\", 42)").unwrap();

        let biscuit = builder.build(&keypair).unwrap();

        let mut authorizer = biscuit.authorizer().unwrap();
        // Biscuit doesn't have support for querying for a single string, so, we do this
        // instead.
        let res: (String, i64) = authorizer
            .query_exactly_one("data($name, 0) <- user($name, $id)")
            .unwrap();
        assert_eq!(res.0, "John Doe");
        assert_eq!(res.1, 0);
    }

    #[tokio::test]
    async fn test_prop_db() {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("test").use_db("test").await.unwrap();
        db.query(
            "DEFINE TABLE OVERWRITE properties TYPE NORMAL SCHEMAFULL PERMISSIONS NONE; DEFINE \
             FIELD id ON properties TYPE { class: string, value: string } PERMISSIONS FULL;",
        )
        .await
        .unwrap();
        let _: Vec<Property> = db
            .insert("properties")
            .content(Property {
                class: Class::Type,
                value: "test".to_string(),
            })
            .await
            .unwrap();
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
}
