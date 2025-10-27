use chrono::{DateTime, Utc};
use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort, async_trait, call};
use salvo::{
    Depot, Response,
    oapi::extract::JsonBody,
    prelude::{StatusCode, StatusError, ToSchema, endpoint},
};
use serde::{Deserialize, Serialize};
use snafu::prelude::*;
use surrealdb::{RecordId, Surreal, engine::local::Db};
use tracing::{debug, error};

use crate::{
    db::{
        ItemID, UserID,
        model::{Class, Property, Source, WorkshopItemProperties},
    },
    web::auth,
};

pub type Result<T, E = Error> = std::result::Result<T, E>;
pub type Error = StatusError;

#[derive(Debug, Snafu)]
#[non_exhaustive]
#[snafu(visibility(pub(crate)))]
enum InnerError {
    #[snafu(display("Invalid vote score"))]
    InvalidVoteScore,
    #[snafu(display("Bad request: {msg}"))]
    BadRequest {
        msg: String,
    },
    Conflict,
    Unauthorized,
    InternalError,
}

impl InnerError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            InnerError::InvalidVoteScore | InnerError::BadRequest { .. } => StatusCode::BAD_REQUEST,
            InnerError::Conflict => StatusCode::CONFLICT,
            InnerError::Unauthorized => StatusCode::UNAUTHORIZED,
            InnerError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<InnerError> for StatusError {
    fn from(value: InnerError) -> Self {
        let mut error = StatusError::internal_server_error();
        error.code = value.status_code();
        error.name = value
            .status_code()
            .canonical_reason()
            .unwrap_or_default()
            .to_string();
        error.brief = value.to_string();
        error.detail = value.backtrace().map(std::string::ToString::to_string);
        error
    }
}

/// Actor responsible for handling workshop item properties operations
/// such as creating a new property and voting on properties.
pub struct PropertiesActor;

/// Messages handled by `PropertiesActor`.
pub enum PropertiesMessage {
    /// Create or relate a new property to an item.
    NewProperty(NewProperty, String, RpcReplyPort<Result<()>>),
    /// Cast or update a vote for a property.
    Vote(VoteData, String, RpcReplyPort<Result<()>>),
    /// Remove a user's vote for a property.
    Remove(VoteData, String, RpcReplyPort<Result<()>>),
}

/// Shared state for `PropertiesActor`.
pub struct PropertiesState {
    database: Surreal<Db>,
}

/// Actor initialization arguments.
pub struct PropertiesArgs {
    pub database: Surreal<Db>,
}

use std::sync::OnceLock;

use snafu::ErrorCompat;

static PROPERTIES_ACTOR: OnceLock<ActorRef<PropertiesMessage>> = OnceLock::new();

#[async_trait]
impl Actor for PropertiesActor {
    type Arguments = PropertiesArgs;
    type Msg = PropertiesMessage;
    type State = PropertiesState;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        PROPERTIES_ACTOR.get_or_init(|| myself);
        Ok(PropertiesState {
            database: args.database,
        })
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            PropertiesMessage::NewProperty(prop, userid, reply) => {
                let res = new_property_impl(&state.database, prop, userid).await;
                let _ = reply.send(res);
            }
            PropertiesMessage::Vote(data, userid, reply) => {
                let res = vote_impl(&state.database, data, userid).await;
                let _ = reply.send(res);
            }
            PropertiesMessage::Remove(data, userid, reply) => {
                let res = remove_impl(&state.database, data, userid).await;
                let _ = reply.send(res);
            }
        }
        Ok(())
    }
}

/// Add or change a vote for a property.
/// Property must exist; score must be either 1 or -1.
#[endpoint]
pub async fn vote(
    vote_data: JsonBody<VoteData>,
    depot: &mut Depot,
    _resp: &mut Response,
) -> Result<()> {
    let Some(userid) = auth::get_user_from_depot(depot) else {
        return Err(InnerError::Unauthorized.into());
    };
    let actor = PROPERTIES_ACTOR
        .get()
        .cloned()
        .ok_or(InnerError::InternalError)?;
    call!(actor, |reply| PropertiesMessage::Vote(
        vote_data.0,
        userid,
        reply
    ))
    .map_err(|_| InnerError::InternalError)??;
    Ok(())
}

/// Remove a vote previously cast for a property by the current user.
#[endpoint]
pub async fn remove(
    vote_data: JsonBody<VoteData>,
    depot: &mut Depot,
    _resp: &mut Response,
) -> Result<()> {
    let Some(userid) = auth::get_user_from_depot(depot) else {
        return Err(InnerError::Unauthorized.into());
    };
    let actor = PROPERTIES_ACTOR
        .get()
        .cloned()
        .ok_or(InnerError::InternalError)?;
    call!(actor, |reply| PropertiesMessage::Remove(
        vote_data.0,
        userid,
        reply
    ))
    .map_err(|_| InnerError::InternalError)??;
    Ok(())
}
use salvo::Writer;

/// Add a new property with the following rules:
/// - Either entirely new, or an exact match to an existing property.
/// - Likeness checks are done on the value only using Damerau–Levenshtein
///   distance.
#[endpoint]
pub async fn new(
    new_property: JsonBody<NewProperty>,
    depot: &mut Depot,
    _resp: &mut Response,
) -> Result<()> {
    let Some(userid) = auth::get_user_from_depot(depot) else {
        return Err(InnerError::Unauthorized.into());
    };
    let actor = PROPERTIES_ACTOR
        .get()
        .cloned()
        .ok_or(InnerError::InternalError)?;
    call!(actor, |reply| PropertiesMessage::NewProperty(
        new_property.0,
        userid,
        reply
    ))
    .map_err(|_| InnerError::InternalError)??;
    Ok(())
}

async fn vote_impl(db: &Surreal<Db>, vote_data: VoteData, userid: String) -> Result<()> {
    if vote_data.score != 1 && vote_data.score != -1 {
        return Err(InnerError::InvalidVoteScore.into());
    }
    let user = UserID::from(userid);
    let query = db
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
                return {chnaged_score: $changed_score, score: $score};
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
            Err(InnerError::BadRequest {
                msg: "Invalid vote".into(),
            }
            .into())
        }
        Err(e) => {
            error!(?e, "vote query error");
            Err(InnerError::InternalError.into())
        }
    }
}

async fn remove_impl(db: &Surreal<Db>, vote_data: VoteData, userid: String) -> Result<()> {
    let user = UserID::from(userid);
    let result = db
        .query("BEGIN TRANSACTION;")
        .query("LET $link = properties:{class: $class, value: $value}")
        .query(
            "let $before = DELETE only votes:{link: $link, user: $user, item: $item} RETURN \
             BEFORE;",
        )
        .query(
            "if $before.score{RETURN UPDATE ONLY workshop_item_properties SET \
             vote_count=math::max([vote_count-1, 0]), upvote_count-=$before.score WHERE in=$item \
             AND out=$link RETURN diff};",
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
            Err(InnerError::BadRequest {
                msg: "Invalid removal".into(),
            }
            .into())
        }
        Err(e) => {
            error!(?e, "vote removal query error");
            Err(InnerError::InternalError.into())
        }
    }
}

async fn new_property_impl(
    db: &Surreal<Db>,
    new_property: NewProperty,
    userid: String,
) -> Result<()> {
    let workshop_id = ItemID::from(new_property.workshop_item).into_recordid();

    let test_prop = Property {
        class: new_property.class,
        value: new_property.value.to_ascii_lowercase(),
    };

    // Basic validation
    if test_prop.value.len() < 4 {
        return Err(InnerError::BadRequest {
            msg: format!(
                "Property cannot be shorter than 4 characters; is {}",
                test_prop.value.len()
            ),
        }
        .into());
    }
    if test_prop.value.len() > 32 {
        return Err(InnerError::BadRequest {
            msg: format!(
                "Property cannot be longer than 32 characters; is {}",
                test_prop.value.len()
            ),
        }
        .into());
    }
    if !test_prop
        .value
        .chars()
        .all(|c| c.is_alphabetic() || c == ' ')
    {
        return Err(InnerError::BadRequest {
            msg: "Property value must be alphabetic characters only".into(),
        }
        .into());
    }

    // Similarity and existence checks
    let prop_exists = {
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
        let res = match query.await {
            Ok(r) => r,
            Err(e) => {
                error!(?e, "query failed");
                return Err(InnerError::InternalError.into());
            }
        };
        let mut res = match res.check() {
            Ok(r) => r,
            Err(e) => {
                error!(?e, "check failed");
                return Err(InnerError::InternalError.into());
            }
        };

        let similar_properties = res.take::<Vec<Property>>(0).unwrap_or_default();
        if !similar_properties.is_empty() && !similar_properties.contains(&test_prop) {
            // return similar entries; treat as conflict
            debug!(?similar_properties, "Similar properties exist");
            return Err(InnerError::Conflict.into());
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
        .bind(("note", new_property.note))
        .bind((
            "source",
            Source::<RecordId>::User(UserID::from(userid).into()),
        ))
        .await
        .map(surrealdb::Response::check)
    {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(surrealdb::Error::Db(surrealdb::err::Error::IndexExists { .. }))) => {
            Err(InnerError::Conflict.into())
        }
        Ok(Err(other)) => {
            error!(?other, "unexpected DB error");
            Err(InnerError::InternalError.into())
        }
        Err(e) => {
            error!(?e, "query error");
            Err(InnerError::InternalError.into())
        }
    }
}

/// Crowdsourced metadata for an item, private version.
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct NewProperty {
    pub class: Class,
    pub value: String,
    /// Reasoning or justification for an inclusion
    pub note: Option<String>,
    /// Workshop item identifier this property relates to
    pub workshop_item: String,
}

/// Payload for casting a vote on an item property.
#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct VoteData {
    /// Workshop item identifier
    pub item: String,
    /// Property class
    pub class: Class,
    /// Property value
    pub value: String,
    /// Must be -1 or 1
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

    use crate::db::model::{Class, Property};

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
}
