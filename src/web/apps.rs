use ractor::{ActorProcessingErr, RactorErr, call};
use reqwest::StatusCode;
use salvo::{
    http::StatusError,
    oapi::{
        endpoint,
        extract::{JsonBody, QueryParam},
    },
    prelude::*,
};
use snafu::{ErrorCompat, Snafu};

use crate::{
    db::{
        apps_actor::{APPS_ACTOR, AppsMsg},
        model::App,
    },
    domain::apps::AppError,
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
    Unavailable,
    NotFound,
}

impl InnerError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            InnerError::BadRequest { .. } => StatusCode::BAD_REQUEST,
            InnerError::Conflict => StatusCode::CONFLICT,
            InnerError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            InnerError::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
            InnerError::NotFound => StatusCode::NOT_FOUND,
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
impl From<AppError> for InnerError {
    fn from(value: AppError) -> Self {
        match value {
            AppError::BadRequest { msg } => Self::BadRequest { msg },
            AppError::Conflict => Self::Conflict,
            AppError::Internal => Self::InternalError,
            AppError::NotFound => Self::NotFound,
        }
    }
}

#[endpoint]
pub async fn list_available() -> Result<Json<Vec<App>>> {
    let actor = APPS_ACTOR.get().ok_or(InnerError::Unavailable)?;
    let apps = call!(actor, AppsMsg::ListAvailable)
        .map_err(InnerError::from)?
        .map_err(InnerError::from)?;
    Ok(Json(apps))
}

#[endpoint]
pub async fn upsert(app: JsonBody<App>) -> Result<()> {
    let actor = APPS_ACTOR.get().ok_or(InnerError::Unavailable)?;
    call!(actor, |reply| AppsMsg::Upsert(app.0, reply))
        .map_err(InnerError::from)?
        .map_err(InnerError::from)?;
    Ok(())
}
#[endpoint]
pub async fn remove(id: QueryParam<u32, true>) -> Result<()> {
    let actor = APPS_ACTOR.get().ok_or(InnerError::Unavailable)?;
    call!(actor, |reply| AppsMsg::Remove(*id, reply))
        .map_err(InnerError::from)?
        .map_err(InnerError::from)?;
    Ok(())
}

#[endpoint]
pub async fn list() -> Result<Json<Vec<App>>> {
    let actor = APPS_ACTOR.get().ok_or(InnerError::Unavailable)?;
    let apps = call!(actor, AppsMsg::List)
        .map_err(InnerError::from)?
        .map_err(InnerError::from)?;
    Ok(Json(apps))
}

#[endpoint]
pub async fn get(id: QueryParam<u32, true>) -> Result<Json<App>> {
    let actor = APPS_ACTOR.get().ok_or(InnerError::Unavailable)?;
    let app = call!(actor, |reply| AppsMsg::Get(*id, reply))
        .map_err(InnerError::from)?
        .map_err(InnerError::from)?;
    Ok(Json(app))
}
