use ractor::{ActorProcessingErr, RactorErr, call};
use salvo::{
    Depot, Writer,
    oapi::extract::JsonBody,
    prelude::{Json, StatusCode, StatusError, endpoint},
};
use snafu::{ErrorCompat, prelude::*};

use crate::{
    db::{
        admin_actor::{ADMIN_ACTOR, AdminMsg},
        model::{Property, User, WorkshopItemProperties},
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

#[endpoint]
pub async fn get_users(_: &mut Depot) -> Result<Json<Vec<User<String>>>> {
    let actor = ADMIN_ACTOR
        .get()
        .cloned()
        .ok_or(InnerError::InternalError)?;
    let users: Vec<User<String>> = call!(actor, |reply| AdminMsg::ListUsers(reply))
        .map_err(InnerError::from)?
        .map_err(InnerError::from)?;
    Ok(Json(users))
}

#[endpoint]
pub async fn patch_user(data: JsonBody<PatchUserData>, _: &mut Depot) -> Result<()> {
    let actor = ADMIN_ACTOR
        .get()
        .cloned()
        .ok_or(InnerError::InternalError)?;
    call!(actor, |reply| AdminMsg::PatchUser(data.0, reply))
        .map_err(InnerError::from)?
        .map_err(InnerError::from)?;
    Ok(())
}

#[endpoint]
pub async fn get_workshop_item_properties(
    _: &mut Depot,
) -> Result<Json<Vec<WorkshopItemProperties<String, Property>>>> {
    let actor = ADMIN_ACTOR
        .get()
        .cloned()
        .ok_or(InnerError::InternalError)?;
    let list = call!(actor, |reply| AdminMsg::ListWorkshopItemProperties(reply))
        .map_err(InnerError::from)?
        .map_err(InnerError::from)?;
    Ok(Json(list))
}

#[endpoint]
pub async fn patch_workshop_item_properties(
    data: JsonBody<PatchRelationshipData>,
    _: &mut Depot,
) -> Result<()> {
    let actor = ADMIN_ACTOR
        .get()
        .cloned()
        .ok_or(InnerError::InternalError)?;
    call!(actor, |reply| AdminMsg::PatchWorkshopItemProperty(
        data.0, reply
    ))
    .map_err(InnerError::from)?
    .map_err(InnerError::from)?;
    Ok(())
}
