use std::{
    mem,
    str::FromStr,
    sync::{Arc, OnceLock},
    time::SystemTime,
};

use biscuit_auth::{
    Authorizer, Biscuit, KeyPair,
    builder_ext::AuthorizerExt,
    macros::{authorizer, biscuit},
};
use chrono::{NaiveDateTime, TimeDelta, Utc};
use multimap::MultiMap;
use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort, async_trait, call};
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
use tracing::{debug, error};

use crate::{
    app_config::BiscuitConfig,
    db::{UserID, model::User},
    steam::steam_user_actor::SteamUserMsg,
};

static AUTH_ACTOR: OnceLock<ActorRef<AuthMessage>> = OnceLock::new();

pub type Result<T, E = Error> = std::result::Result<T, E>;
pub type Error = StatusError;
const STEAM_DISCOVERY: &str = "https://steamcommunity.com/openid/";

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
    InternalError,
    Unauthorized,
}

impl InnerError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            InnerError::SelfValidationFailed | InnerError::PeerValidationFailed => {
                StatusCode::FORBIDDEN
            }
            InnerError::QueryingDiscovery
            | InnerError::DiscoveryBadResponse
            | InnerError::DeserializingDiscovery
            | InnerError::InfoAlreadySet
            | InnerError::ExpectedInfoReady
            | InnerError::BuildingURI
            | InnerError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            InnerError::Unauthorized => StatusCode::UNAUTHORIZED,
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

#[endpoint]
pub async fn redirect_to_steam_auth(req: &mut Request, resp: &mut Response) -> Result<()> {
    let location = req
        .query::<String>("location")
        .ok_or(InnerError::SelfValidationFailed)?;
    let actor = AUTH_ACTOR
        .get()
        .cloned()
        .ok_or(InnerError::InternalError)
        .inspect_err(|e| error!(?e, "{}:{}", file!(), line!()))?;
    let steam_auth_url = call!(actor, AuthMessage::GetAuthUrl, location)
        .map_err(|_| InnerError::InternalError)
        .inspect_err(|e| error!(?e, "{}:{}", file!(), line!()))?;

    resp.render(Redirect::found(steam_auth_url));

    Ok(())
}

#[endpoint]
pub async fn verify_token_from_steam(req: &mut Request, response: &mut Response) -> Result<()> {
    // Pull this out first because it'll likely be gone after the take.
    let redirect_to = req
        .query::<String>("location")
        .ok_or(InnerError::SelfValidationFailed)?;
    let actor = AUTH_ACTOR.get().cloned().ok_or(InnerError::InternalError)?;
    let map = mem::take(req.queries_mut());
    let token = call!(actor, |reply| {
        AuthMessage::VerifySteamResponse(map, reply)
    })
    .map_err(|_| InnerError::InternalError)?;

    response.add_cookie(
        Cookie::build(("token", token))
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

    response.render(Redirect::found(redirect_to));
    Ok(())
}

/// Instructs the client to clear the cookies for the site, functioning as
/// logout. Done here because JS can't access the tokens we use.
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
pub async fn validate_biscuit_token(req: &mut Request, depot: &mut Depot) -> Result<()> {
    match req.cookie("token") {
        None => Err(InnerError::Unauthorized)?,
        Some(token) => {
            let actor = AUTH_ACTOR.get().cloned().ok_or(InnerError::InternalError)?;
            let authorizer = call!(actor, |reply| {
                AuthMessage::ValidateToken(token.clone(), reply)
            })
            .map_err(|_| InnerError::InternalError)??;

            depot.inject::<Authorizer>(authorizer);
            Ok(())
        }
    }
}

#[endpoint]
pub async fn enforce_admin(depot: &mut Depot) -> Result<()> {
    match get_user_from_depot(depot) {
        None => Err(InnerError::Unauthorized)?,
        Some(userid) => {
            let actor = AUTH_ACTOR.get().cloned().ok_or(InnerError::InternalError)?;
            let admin = call!(actor, |reply| { AuthMessage::IsAdmin(userid, reply) })
                .map_err(|_| InnerError::InternalError)??;

            if admin {
                Ok(())
            } else {
                Err(InnerError::Unauthorized)?
            }
        }
    }
}
#[endpoint]
pub async fn validate_opt(req: &mut Request, depot: &mut Depot) -> Result<()> {
    if req.cookie("token").is_some() {
        validate_biscuit_token::validate_biscuit_token(req, depot).await?;
    }
    Ok(())
}
/// Returns the user id of the current user, if any.
pub fn get_user_from_depot(depot: &mut Depot) -> Option<String> {
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

pub struct AuthActor {}
pub enum AuthMessage {
    GetAuthUrl(String, RpcReplyPort<String>),
    VerifySteamResponse(MultiMap<String, String>, RpcReplyPort<String>),
    ValidateToken(Cookie<'static>, RpcReplyPort<Result<Authorizer>>),
    IsAdmin(String, RpcReplyPort<Result<bool>>),
}
pub struct AuthState {
    open_id_info: Info,
    database: Surreal<Db>,
    base_url: Arc<String>,
    biscuit: Arc<BiscuitConfig>,
    steam_user_actor_ref: ActorRef<SteamUserMsg>,
}
pub struct AuthArgs {
    pub database: Surreal<Db>,
    pub client: Client,
    pub base_url: Arc<String>,
    pub biscuit: Arc<BiscuitConfig>,
    pub steam_user_actor_ref: ActorRef<SteamUserMsg>,
}
#[async_trait]
impl Actor for AuthActor {
    type Arguments = AuthArgs;
    type Msg = AuthMessage;
    type State = AuthState;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        AUTH_ACTOR.get_or_init(|| myself);
        Ok(Self::State {
            open_id_info: AuthActor::discover_openid_info(&args.client).await?,
            database: args.database,
            base_url: args.base_url.clone(),
            biscuit: args.biscuit.clone(),
            steam_user_actor_ref: args.steam_user_actor_ref,
        })
    }

    // This is our main message handler
    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            AuthMessage::GetAuthUrl(location, reply_port) => {
                if reply_port
                    .send(AuthActor::get_auth_url(state, &location)?)
                    .is_err()
                {
                    error!(message = "GetAuthUrl", "Failed to reply to message");
                }
            }
            AuthMessage::VerifySteamResponse(map, reply_port) => {
                if reply_port
                    .send(AuthActor::verify_steam_response(map, state).await?)
                    .is_err()
                {
                    error!(
                        message = "VerifySteamResponse",
                        "Failed to reply to message"
                    );
                }
            }
            AuthMessage::ValidateToken(cookie, reply_port) => {
                if reply_port
                    .send(AuthActor::validate_cookie(&state.biscuit, cookie.value()))
                    .is_err()
                {
                    error!(message = "ValidateToken", "Failed to reply to message");
                }
            }
            AuthMessage::IsAdmin(userid, reply_port) => {
                if reply_port
                    .send(AuthActor::is_admin(&state.database, userid).await)
                    .is_err()
                {
                    error!(message = "IsAdmin", "Failed to reply to message");
                }
            }
        }
        Ok(())
    }
}

impl AuthActor {
    async fn is_admin(db: &Surreal<Db>, userid: String) -> Result<bool> {
        match db
            .query("SELECT admin FROM $user")
            .bind(("user", UserID::from(userid).into_recordid()))
            .await
            .map(surrealdb::Response::check)
        {
            Err(_) | Ok(Err(_)) => Err(InnerError::InternalError)?,
            Ok(Ok(mut db_response)) => Ok(db_response.take("admin").ok().flatten() == Some(true)),
        }
    }

    fn validate_cookie(config: &BiscuitConfig, token: &str) -> Result<Authorizer> {
        let keypair = &KeyPair::from(&config.private_key);
        let Ok(token) = Biscuit::from_base64(token, keypair.public()) else {
            return Err(InnerError::Unauthorized)?;
        };
        let Ok(mut authorizer) = authorizer!("").time().allow_all().build(&token) else {
            return Err(InnerError::Unauthorized)?;
        };

        if let Err(e) = authorizer.authorize() {
            debug!(error = ?e, "Failed to authorize");
            return Err(InnerError::Unauthorized)?;
        }
        Ok(authorizer)
    }

    fn get_auth_url(state: &AuthState, location: &str) -> Result<String> {
        let mut url =
            Url::from_str(&state.open_id_info.uri).map_err(|_| InnerError::BuildingURI)?;
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
                &AuthActor::redirect_url(&state.base_url, location),
            )
            .append_pair("openid.realm", state.base_url.as_str())
            .finish();

        Ok(url.to_string())
    }

    fn redirect_url(base: &Arc<String>, location: &str) -> String {
        String::clone(base) + "/api/verify?location=" + location
    }

    async fn discover_openid_info(client: &Client) -> Result<Info> {
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
        Ok(Info {
            r#type: doc.xrd.service.r#type,
            uri: doc.xrd.service.uri,
        })
    }

    async fn verify_steam_response(
        map: MultiMap<String, String>,
        state: &mut AuthState,
    ) -> Result<String> {
        {
            if (map.get("openid.ns").map(String::as_str))
                != (Some(
                    &state.open_id_info.r#type[0..state
                        .open_id_info
                        .r#type
                        .len()
                        .saturating_sub(b"/server".len())],
                ))
            {
                return Err(InnerError::SelfValidationFailed)?;
            }

            if (map.get("openid.op_endpoint")) != (Some(&state.open_id_info.uri)) {
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

        if let Ok(id) = user_id.parse::<u64>() {
            let _ = state
                .steam_user_actor_ref
                .send_message(SteamUserMsg::Fetch(id));
        }

        let keypair = &KeyPair::from(&state.biscuit.private_key);

        let biscuit: Biscuit = biscuit!(
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

        {
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
            for (i, error) in state
                .database
                .query(stmt)
                .await
                .map_err(|_| InnerError::PeerValidationFailed)?
                .take_errors()
            {
                error!(i, ?error, "verification failed");
            }
        }
        Ok(based)
    }
}
