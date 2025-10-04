use std::{str::FromStr, sync::Arc, time::SystemTime};

use biscuit_auth::{
    Authorizer, Biscuit, KeyPair,
    builder_ext::AuthorizerExt,
    macros::{authorizer, biscuit},
};
use chrono::{NaiveDateTime, TimeDelta, Utc};
use itertools::Itertools;
use reqwest::{Client, Url};
use salvo::{
    Depot, Request, Response,
    http::{
        HeaderValue,
        cookie::{Cookie, SameSite, time::Duration},
    },
    prelude::{Redirect, StatusCode, StatusError, endpoint},
};
use serde::{Deserialize, Serialize};
use serde_xml_rs::from_str;
use snafu::{ErrorCompat, prelude::*};
use surrealdb::{
    Surreal,
    engine::local::Db,
    sql::{Data, Operator, Value, statements::InsertStatement, to_value},
    syn::idiom,
};
use tokio::sync::OnceCell;
use tracing::{debug, error};

use crate::{
    app_config::Config,
    db::{UserID, model::User},
};

pub type Result<T, E = Error> = std::result::Result<T, E>;
pub type Error = StatusError;
const STEAM_DISCOVERY: &str = "https://steamcommunity.com/openid/";
static OPENID_INFO: OnceCell<Info> = OnceCell::const_new();

const TOKEN_LIFETIME: Duration = Duration::days(30);

struct Info {
    r#type: String,
    uri: String,
}

#[derive(Debug, Snafu)]
#[non_exhaustive]
#[snafu(visibility(pub(crate)))]
enum InnerError {
    QueryingDiscovery,
    DiscoveryBadResponse,
    DeserializingDiscovery,
    InfoAlreadySet,
    ExpectedInfoReady,
    BuildingURI,
    SelfValidationFailed,
    PeerValidationFailed,
}

impl InnerError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            InnerError::QueryingDiscovery => StatusCode::INTERNAL_SERVER_ERROR,
            InnerError::DiscoveryBadResponse => StatusCode::INTERNAL_SERVER_ERROR,
            InnerError::DeserializingDiscovery => StatusCode::INTERNAL_SERVER_ERROR,
            InnerError::InfoAlreadySet => StatusCode::INTERNAL_SERVER_ERROR,
            InnerError::ExpectedInfoReady => StatusCode::INTERNAL_SERVER_ERROR,
            InnerError::BuildingURI => StatusCode::INTERNAL_SERVER_ERROR,
            InnerError::SelfValidationFailed => StatusCode::FORBIDDEN,
            InnerError::PeerValidationFailed => StatusCode::FORBIDDEN,
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

pub async fn get_url(client: Client, config: &Config, location: &str) -> Result<String> {
    if !OPENID_INFO.initialized() {
        let response = client
            .get(STEAM_DISCOVERY)
            .send()
            .await
            .map_err(|_| InnerError::QueryingDiscovery)?;
        let response_text = response
            .text()
            .await
            .map_err(|_| InnerError::DiscoveryBadResponse)?;

        let doc: Xrds = from_str(&response_text).map_err(|_| InnerError::DeserializingDiscovery)?;
        OPENID_INFO
            .set(Info {
                r#type: doc.xrd.service.r#type,
                uri: doc.xrd.service.uri,
            })
            .map_err(|_| InnerError::InfoAlreadySet)?;
    }
    let info = OPENID_INFO.get().ok_or(InnerError::ExpectedInfoReady)?;
    let mut url = Url::from_str(&info.uri).map_err(|_| InnerError::BuildingURI)?;
    url.query_pairs_mut()
        .append_pair("openid.mode", "checkid_setup")
        .append_pair("openid.ns", "http://specs.openid.net/auth/2.0")
        .append_pair(
            "openid.claimed_id",
            "http://specs.openid.net/auth/2.0/identifier_select",
        )
        .append_pair(
            "openid.identity",
            "http://specs.openid.net/auth/2.0/identifier_select",
        )
        .append_pair(
            "openid.return_to",
            &redirect_url(&config.base_url, location),
        )
        .append_pair("openid.realm", config.base_url.as_str())
        .finish();

    Ok(url.to_string())
}

fn redirect_url(base: &Arc<String>, location: &str) -> String {
    String::clone(base) + "/api/verify?location=" + location
}

#[endpoint]
pub async fn redirect(req: &mut Request, resp: &mut Response, depot: &mut Depot) -> Result<()> {
    let client = Client::new();
    let config = depot
        .obtain::<Arc<Config>>()
        .map_err(|_| InnerError::ExpectedInfoReady)?;
    let location = req
        .query::<&str>("location")
        .ok_or(InnerError::SelfValidationFailed)?;
    let url = get_url(client, config, location).await?;
    resp.render(Redirect::found(url));

    Ok(())
}

#[endpoint]
pub async fn verify(req: &mut Request, response: &mut Response, depot: &mut Depot) -> Result<()> {
    let map = req.queries();
    {
        let info = OPENID_INFO.get().ok_or(InnerError::ExpectedInfoReady)?;
        if (map.get("openid.ns").map(String::as_str))
            != (Some(&info.r#type[0..info.r#type.len().saturating_sub(b"/server".len())]))
        {
            return Err(InnerError::SelfValidationFailed)?;
        }

        if (map.get("openid.op_endpoint")) != (Some(&info.uri)) {
            return Err(InnerError::SelfValidationFailed)?;
        }
        if let Some((timestamp, _)) = map
            .get("openid.response_nonce")
            .ok_or(InnerError::SelfValidationFailed)?
            .split_once('Z')
        {
            let timestamp = NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S")
                .map_err(|_| InnerError::SelfValidationFailed)?;
            if timestamp - Utc::now().naive_utc() > TimeDelta::minutes(5) {
                return Err(InnerError::SelfValidationFailed)?;
            }
        } else {
            return Err(InnerError::SelfValidationFailed)?;
        }
    }

    let mut url = Url::from_str("https://steamcommunity.com/openid/login").unwrap();

    for item in map
        .get("openid.signed")
        .ok_or(InnerError::SelfValidationFailed)?
        .split(',')
    {
        let key = format!("openid.{item}");
        let val = map.get(&key).ok_or(InnerError::SelfValidationFailed)?;
        url.query_pairs_mut().append_pair(&key, val).finish();
    }
    url.query_pairs_mut()
        .append_pair(
            "openid.sig",
            map.get("openid.sig")
                .ok_or(InnerError::SelfValidationFailed)?,
        )
        .append_pair(
            "openid.ns",
            map.get("openid.ns")
                .ok_or(InnerError::SelfValidationFailed)?,
        )
        .append_pair("openid.mode", "check_authentication")
        .finish();

    let resp = reqwest::get(url)
        .await
        .map_err(|_| InnerError::PeerValidationFailed)?;
    let text = resp
        .text()
        .await
        .map_err(|_| InnerError::PeerValidationFailed)?;

    if text != "ns:http://specs.openid.net/auth/2.0\nis_valid:true\n" {
        return Err(InnerError::PeerValidationFailed)?;
    }

    let user_id = map
        .get("openid.identity")
        .ok_or(InnerError::PeerValidationFailed)?
        .rsplit('/')
        .next()
        .ok_or(InnerError::PeerValidationFailed)?;

    let config = depot
        .obtain::<Arc<Config>>()
        .map_err(|_| InnerError::ExpectedInfoReady)?;
    let keypair = &KeyPair::from(&config.biscuit.private_key);

    let biscuit: biscuit_auth::Biscuit = biscuit!(
        r#"
          user({user_id});
          check if time($time), $time <= {expires};
    "#,
        expires = SystemTime::now() + TOKEN_LIFETIME
    )
    .build(keypair)
    .map_err(|_| InnerError::PeerValidationFailed)?;

    let based = biscuit
        .to_base64()
        .map_err(|_| InnerError::PeerValidationFailed)?;

    response.add_cookie(
        Cookie::build(("token", based))
            .max_age(TOKEN_LIFETIME)
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Strict)
            .path("/")
            .build(),
    );

    response.add_cookie(
        Cookie::build("token_set")
            .max_age(TOKEN_LIFETIME)
            .secure(true)
            .same_site(SameSite::Strict)
            .path("/")
            .build(),
    );
    {
        let db = depot
            .obtain::<Surreal<Db>>()
            .map_err(|_| InnerError::ExpectedInfoReady)?;
        let user = User {
            id: UserID::from(user_id.to_owned()).into_recordid(),
            admin: false,
            banned: false,
            last_logged_in: Utc::now(),
        };
        let mut stmt = InsertStatement::default();
        stmt.into = Some(Value::Table("users".into()));
        stmt.data = Data::SingleExpression(
            to_value(user.clone()).map_err(|_| InnerError::PeerValidationFailed)?,
        );
        stmt.update = Some(Data::UpdateExpression(vec![(
            idiom("last_logged_in").map_err(|_| InnerError::PeerValidationFailed)?,
            Operator::Equal,
            Utc::now().into(),
        )]));
        for (i, error) in db
            .query(stmt)
            .await
            .map_err(|_| InnerError::PeerValidationFailed)?
            .take_errors()
        {
            error!(i, ?error, "verification failed");
        }
    }

    let redirect_to = req
        .query::<&str>("location")
        .ok_or(InnerError::SelfValidationFailed)?;
    response.render(Redirect::found(redirect_to));
    Ok(())
}

#[endpoint]
pub async fn invalidate(req: &mut Request, response: &mut Response) -> Result<()> {
    response
        .headers
        .insert("Clear-Site-Data", HeaderValue::from_static("\"cookies\""));
    let redirect_to = req
        .query::<&str>("location")
        .ok_or(InnerError::SelfValidationFailed)?;
    response.render(Redirect::found(redirect_to));
    Ok(())
}

#[endpoint]
pub async fn validate(req: &mut Request, depot: &mut Depot, response: &mut Response) {
    match req.cookie("token") {
        None => {
            response.status_code(StatusCode::UNAUTHORIZED);
        }
        Some(token) => {
            let Ok(config) = depot.obtain::<Arc<Config>>() else {
                response.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                return;
            };
            let keypair = &KeyPair::from(&config.biscuit.private_key);
            let Ok(token) = Biscuit::from_base64(token.value(), keypair.public()) else {
                response.status_code(StatusCode::UNAUTHORIZED);
                return;
            };
            let Ok(mut authorizer) = authorizer!("").time().allow_all().build(&token) else {
                response.status_code(StatusCode::UNAUTHORIZED);
                return;
            };

            if let Err(e) = authorizer.authorize() {
                debug!(error = ?e, "Failed to authorize");
                response.status_code(StatusCode::UNAUTHORIZED);
                return;
            }

            depot.inject(authorizer);
        }
    }
}

#[endpoint]
pub async fn enforce_admin(depot: &mut Depot, response: &mut Response) {
    match get_user(depot) {
        None => {
            response.status_code(StatusCode::UNAUTHORIZED);
        }
        Some(userid) => {
            match depot
                .obtain::<Surreal<Db>>()
                .expect("db")
                .query("SElECT admin FROM $user")
                .bind(("user", UserID::from(userid).into_recordid()))
                .await
                .map(surrealdb::Response::check)
            {
                Err(error) => {
                    error!(?error, "running admin query");
                    response.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                }
                Ok(Err(error)) => {
                    error!(?error, "checking admin query");
                    response.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                }
                Ok(Ok(mut db_response)) => {
                    if db_response.take("admin").ok().flatten() != Some(true) {
                        response.status_code(StatusCode::UNAUTHORIZED);
                    }
                }
            }
        }
    }
}
#[endpoint]
pub async fn validate_opt(req: &mut Request, depot: &mut Depot, response: &mut Response) {
    if req.cookie("token").is_some() {
        validate::validate(req, depot, response).await;
    }
}

pub fn get_user(depot: &mut Depot) -> Option<String> {
    let authorizer = depot.obtain_mut::<Authorizer>().ok()?;
    let (userid, _): (String, i64) = authorizer
        .query_exactly_one("data($user, 0) <- user($user)")
        .ok()?;
    Some(userid)
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Xrds {
    #[serde(rename = "XRD")]
    xrd: Xrd,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Xrd {
    #[serde(rename = "Service")]
    service: Service,
}
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Service {
    #[serde(rename = "Type")]
    r#type: String,
    #[serde(rename = "URI")]
    uri: String,
}
