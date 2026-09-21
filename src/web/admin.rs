use ractor::{ActorProcessingErr, RactorErr, call};
use salvo::{
    Writer,
    oapi::extract::JsonBody,
    prelude::{Json, StatusCode, StatusError, endpoint},
};
use snafu::{ErrorCompat, prelude::*};

use crate::{
    db::{
        admin_actor::{ADMIN_ACTOR, AdminRequest},
        model::{ExternalUser, ExternalWorkshopItemProperties, InternalUser},
    },
    domain::admin::{AdminError, PatchRelationshipData, PatchUserData},
};

pub type Result<T, E = Error> = std::result::Result<T, E>;
pub type Error = StatusError;

#[derive(Debug, Snafu)]
#[non_exhaustive]
#[snafu(visibility(pub(crate)))]
enum InnerError {
    #[snafu(display("Bad request: {msg}"))]
    BadRequest {
        msg: String,
    },
    Conflict,
    InternalError,
}

impl InnerError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            InnerError::BadRequest { .. } => StatusCode::BAD_REQUEST,
            InnerError::Conflict => StatusCode::CONFLICT,
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
        error.detail = value.backtrace().map(ToString::to_string);
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
impl From<AdminError> for InnerError {
    fn from(value: AdminError) -> Self {
        match value {
            AdminError::BadRequest { msg } => Self::BadRequest { msg },
            AdminError::Conflict => Self::Conflict,
            AdminError::Internal => Self::InternalError,
        }
    }
}

#[tracing::instrument(level = "debug", name = "GET /api/admin/users")]
#[endpoint]
pub async fn get_users() -> Result<Json<Vec<ExternalUser>>, StatusError> {
    let actor = ADMIN_ACTOR
        .get()
        .cloned()
        .ok_or(InnerError::InternalError)?;
    let users: Vec<InternalUser> = call!(actor, AdminRequest::ListUsers)
        .map_err(InnerError::from)?
        .map_err(InnerError::from)?;
    let users: Vec<ExternalUser> = users
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| InnerError::InternalError)?;
    Ok(Json(users))
}

#[tracing::instrument(level = "debug", name = "PUT /api/admin/users", skip(data), fields(user.id = ?data.0.id))]
#[endpoint]
pub async fn patch_user(data: JsonBody<PatchUserData>) -> Result<()> {
    let actor = ADMIN_ACTOR
        .get()
        .cloned()
        .ok_or(InnerError::InternalError)?;
    call!(actor, |reply| AdminRequest::PatchUser(data.0, reply))
        .map_err(InnerError::from)?
        .map_err(InnerError::from)?;
    Ok(())
}

#[tracing::instrument(level = "debug", name = "GET /api/admin/properties")]
#[endpoint]
pub async fn get_workshop_item_properties() -> Result<Json<Vec<ExternalWorkshopItemProperties>>> {
    let actor = ADMIN_ACTOR
        .get()
        .cloned()
        .ok_or(InnerError::InternalError)?;
    let list = call!(actor, AdminRequest::ListWorkshopItemProperties)
        .map_err(InnerError::from)?
        .map_err(InnerError::from)?
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<_, _>>()
        .map_err(|_| InnerError::InternalError)?;
    Ok(Json(list))
}

#[tracing::instrument(level = "debug", name = "PUT /api/admin/properties", skip(data))]
#[endpoint]
pub async fn patch_workshop_item_properties(data: JsonBody<PatchRelationshipData>) -> Result<()> {
    let actor = ADMIN_ACTOR
        .get()
        .cloned()
        .ok_or(InnerError::InternalError)?;
    call!(actor, |reply| AdminRequest::PatchWorkshopItemProperty(
        data.0, reply
    ))
    .map_err(InnerError::from)?
    .map_err(InnerError::from)?;
    Ok(())
}
