use ractor::{ActorProcessingErr, RactorErr, call};
use reqwest::StatusCode;
use salvo::{
    http::StatusError,
    oapi::{
        endpoint,
        extract::{JsonBody, PathParam, QueryParam},
    },
    prelude::*,
};
use snafu::{ErrorCompat, Snafu};

use crate::{
    db::{
        AppID,
        apps_actor::{APPS_ACTOR, AppsRequest},
        model::ExternalApp,
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

#[tracing::instrument(level = "debug", name = "GET /api/apps")]
#[endpoint]
pub async fn list_available() -> Result<Json<Vec<ExternalApp>>> {
    let actor = APPS_ACTOR.get().ok_or(InnerError::Unavailable)?;
    let apps = call!(actor, AppsRequest::ListAvailable)
        .map_err(InnerError::from)?
        .map_err(InnerError::from)?;
    let apps: Vec<ExternalApp> = apps
        .into_iter()
        .map(TryFrom::try_from)
        .collect::<Result<_, _>>()
        .map_err(|_| InnerError::InternalError)?;
    Ok(Json(apps))
}

#[tracing::instrument(level = "debug", name = "POST /api/admin/apps", skip(app))]
#[endpoint]
pub async fn upsert(app: JsonBody<ExternalApp>) -> Result<()> {
    let actor = APPS_ACTOR.get().ok_or(InnerError::Unavailable)?;
    call!(actor, |reply| AppsRequest::Upsert(app.0.into(), reply))
        .map_err(InnerError::from)?
        .map_err(InnerError::from)?;
    Ok(())
}
#[tracing::instrument(level = "debug", name = "DELETE /api/admin/app", skip(id), fields(app.id = ?*id))]
#[endpoint]
pub async fn remove(id: QueryParam<AppID, true>) -> Result<()> {
    let actor = APPS_ACTOR.get().ok_or(InnerError::Unavailable)?;
    call!(actor, |reply| AppsRequest::Remove(
        id.into_inner().into(),
        reply
    ))
    .map_err(InnerError::from)?
    .map_err(InnerError::from)?;
    Ok(())
}

#[tracing::instrument(level = "debug", name = "GET /api/admin/apps")]
#[endpoint]
pub async fn list() -> Result<Json<Vec<ExternalApp>>> {
    let actor = APPS_ACTOR.get().ok_or(InnerError::Unavailable)?;
    let apps = call!(actor, AppsRequest::List)
        .map_err(InnerError::from)?
        .map_err(InnerError::from)?;
    let apps: Vec<ExternalApp> = apps
        .into_iter()
        .map(TryFrom::try_from)
        .collect::<Result<_, _>>()
        .map_err(|_| InnerError::InternalError)?;
    Ok(Json(apps))
}

#[tracing::instrument(level = "debug", name = "GET /api/app/{id}", skip(id), fields(app.id = ?*id))]
#[endpoint]
pub async fn get(id: PathParam<AppID>) -> Result<Json<ExternalApp>> {
    let actor = APPS_ACTOR.get().ok_or(InnerError::Unavailable)?;
    let app = call!(actor, |reply| AppsRequest::Get(
        id.into_inner().into(),
        reply
    ))
    .map_err(InnerError::from)?
    .map_err(InnerError::from)?;
    let app = app.try_into().map_err(|_| InnerError::InternalError)?;
    Ok(Json(app))
}
