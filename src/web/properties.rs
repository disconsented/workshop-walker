use ractor::{call, ActorProcessingErr, RactorErr};
use salvo::{
    oapi::extract::JsonBody, prelude::{endpoint, StatusCode, StatusError},
    Depot,
    Writer,
};
use snafu::{prelude::*, ErrorCompat};

use crate::{
    db::model::Source,
    domain::properties::{NewProperty, PropertiesError, VoteData},
    web::auth,
};
use crate::db::properties_actor::{PropertiesMsg, PROPERTIES_ACTOR};

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

impl From<ActorProcessingErr> for InnerError {
    fn from(_: ActorProcessingErr) -> Self {
        Self::InternalError
    }
}

impl<T> From<RactorErr<T>> for InnerError {
    fn from(_: RactorErr<T>) -> Self {
        Self::InternalError
    }
}

impl From<PropertiesError> for InnerError {
    fn from(value: PropertiesError) -> Self {
        match value {
            PropertiesError::InvalidVoteScore => Self::InvalidVoteScore,
            PropertiesError::BadRequest { msg } => Self::BadRequest { msg },
            PropertiesError::Conflict => Self::Conflict,
            PropertiesError::Internal => Self::InternalError,
        }
    }
}
/// Add or change a vote for a property.
/// Property must exist; score must be either 1 or -1.
#[endpoint]
pub async fn vote(vote_data: JsonBody<VoteData>, depot: &mut Depot) -> Result<()> {
    let Some(userid) = auth::get_user_from_depot(depot) else {
        return Err(InnerError::Unauthorized.into());
    };
    let actor = PROPERTIES_ACTOR
        .get()
        .cloned()
        .ok_or(InnerError::InternalError)?;
    call!(actor, |reply| PropertiesMsg::Vote(
        vote_data.0,
        userid,
        reply
    ))
    .map_err(InnerError::from)?
    .map_err(InnerError::from)?;
    Ok(())
}

/// Remove a vote previously cast for a property by the current user.
#[endpoint]
pub async fn remove(vote_data: JsonBody<VoteData>, depot: &mut Depot) -> Result<()> {
    let Some(userid) = auth::get_user_from_depot(depot) else {
        return Err(InnerError::Unauthorized.into());
    };
    let actor = PROPERTIES_ACTOR
        .get()
        .cloned()
        .ok_or(InnerError::InternalError)?;
    call!(actor, |reply| PropertiesMsg::Remove(
        vote_data.0,
        userid,
        reply
    ))
    .map_err(InnerError::from)?
    .map_err(InnerError::from)?;
    Ok(())
}

/// Add a new property with the following rules:
/// - Either entirely new, or an exact match to an existing property.
/// - Likeness checks are done on the value only using Damerau–Levenshtein
///   distance.
#[endpoint]
pub async fn new(new_property: JsonBody<NewProperty>, depot: &mut Depot) -> Result<()> {
    let Some(userid) = auth::get_user_from_depot(depot) else {
        return Err(InnerError::Unauthorized.into());
    };
    let actor = PROPERTIES_ACTOR
        .get()
        .cloned()
        .ok_or(InnerError::InternalError)?;
    call!(actor, |reply| PropertiesMsg::NewProperty(
        new_property.0,
        Source::User(userid),
        reply
    ))
    .map_err(InnerError::from)?
    .map_err(InnerError::from)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use surrealdb::{engine::local::Mem, Surreal};

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
