use std::sync::OnceLock;

use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort, async_trait, call};
use salvo::{
    Depot, Writer,
    oapi::{endpoint, extract::PathParam},
    prelude::{Json, StatusCode, StatusError},
};
use surrealdb::{RecordId, Surreal, engine::local::Db};
use tracing::{error, instrument};

use crate::{db::model::FullWorkshopItem, web::auth};

static ITEM_ACTOR: OnceLock<ActorRef<ItemMsg>> = OnceLock::new();

pub type Result<T, E = Error> = std::result::Result<T, E>;
pub type Error = StatusError;

#[derive(Debug)]
enum InnerError {
    NotFound,
    InternalError,
}

impl InnerError {
    fn status_code(&self) -> StatusCode {
        match self {
            InnerError::NotFound => StatusCode::NOT_FOUND,
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
        error.brief = format!("{value:?}");
        error.detail = None;
        error
    }
}

pub struct ItemActor;
pub struct ItemState {
    database: Surreal<Db>,
}
pub struct ItemArgs {
    pub database: Surreal<Db>,
}

pub enum ItemMsg {
    Get(
        String,
        Option<String>,
        RpcReplyPort<Result<FullWorkshopItem>>,
    ),
}

#[async_trait]
impl Actor for ItemActor {
    type Arguments = ItemArgs;
    type Msg = ItemMsg;
    type State = ItemState;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        ITEM_ACTOR.get_or_init(|| myself);
        Ok(ItemState {
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
            ItemMsg::Get(id, user, reply) => {
                let res = get_item(&state.database, id, user).await;
                if reply.send(res).is_err() {
                    error!(message = "Get", "Failed to reply to message");
                }
            }
        }
        Ok(())
    }
}

async fn get_item(db: &Surreal<Db>, id: String, user: Option<String>) -> Result<FullWorkshopItem> {
    let id = RecordId::from_table_key("workshop_items", &id);

    let properties = match user {
        Some(user) => format!(
            "filter(|$prop: any| $prop.status == 1 OR $prop.source == {user})[*].{{
                        id: id.id(),
                        in: in.to_string(),
                        out: out.id.{{
                            class,
                            `value`
                        }},
                        source: 'system',
                        status,
                        upvote_count,
                        vote_count,
                        vote_state: votes:{{
                            item: $id,
                            link: out,
                            user: {{ 0 }}
                        }}.score
                    }}"
        ),
        None => "filter(|$prop: any| $prop.status == 1)[*].{
                        id: id.id(),
                        in: in.to_string(),
                        out: out.id.{
                            class,
                            `value`
                        },
                        source: 'system',
                        status,
                        upvote_count,
                        vote_count
                    }"
        .to_string(),
    };
    let query = "SELECT *,type::number(id.id()) as id, type::record('usernames:⟨' + author + \
                 '⟩').{
                id: type::number(id.id()),
                name
            } AS author, tags.{
                id: id.id(),
                app_id,
                display_name
            } AS tags, ->workshop_item_properties."
        .to_string()
        + &properties
        + " AS properties, '' AS description, $id->item_dependencies[*].{
                id: type::number(out.id.id()),
                title: out.title,
                appid: out.appid,
                author: type::record('usernames:⟨' + out.author + '⟩').{
                    id: type::number(id.id()),
                    name
                },
                languages: out.languages,
                last_updated: out.last_updated,
                tags: out.tags.{
                    id: id.to_string(),
                    app_id,
                    display_name
                },
                preview_url: out.preview_url,
                score: out.score,
                description: out.description,
                properties: []
            } AS dependencies, id<-item_dependencies[*].{
                id: type::number(in.id.id()),
                title: in.title,
                appid: in.appid,
                author: type::record('usernames:⟨' + in.author + '⟩').{
                    id: type::number(id.id()),
                    name
                },
                languages: in.languages,
                last_updated: in.last_updated,
                tags: in.tags.{
                    id: id.to_string(),
                    app_id,
                    display_name
                },
                preview_url: in.preview_url,
                score: in.score,
                description: in.description,
                properties: []
            } AS dependants FROM $id;";

    let result: Option<_> = db
        .query(query)
        .bind(("id", id))
        .await
        .inspect_err(|error| error!(message = "get_item", ?error, "Failed to query database"))
        .map_err(|_| InnerError::InternalError)?
        // .inspect(|res| {
        //     dbg!(res);
        // })
        .take(0)
        .inspect_err(|error| error!(message = "get_item", ?error, "Failed to take result"))
        .map_err(|_| InnerError::InternalError)?;
    result.ok_or(Error::from(InnerError::NotFound))
}

/// GET /api/item/{id}
/// Retrieves a full workshop item by id, including dependencies and dependants.
#[endpoint]
#[instrument(skip_all)]
pub async fn get(id: PathParam<String>, depot: &mut Depot) -> Result<Json<FullWorkshopItem>> {
    // Lazily spawn the actor on first use and keep a global reference like auth.rs
    let actor = ITEM_ACTOR.get().cloned().ok_or(InnerError::InternalError)?;

    let user = auth::get_user_from_depot(depot);
    let data = call!(actor, |reply| { ItemMsg::Get(id.0, user, reply) })
        .map_err(|_| InnerError::InternalError)??;
    Ok(Json(data))
}
