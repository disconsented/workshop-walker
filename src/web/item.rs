use std::sync::OnceLock;

use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort, async_trait, call};
use salvo::{
    Depot, Writer,
    oapi::{endpoint, extract::PathParam},
    prelude::{Json, StatusCode, StatusError},
};
use surrealdb::{RecordId, Surreal, engine::local::Db};
use tracing::{error, instrument};

use crate::{
    db::{
        UserID,
        model::{FullWorkshopItem, WorkshopItem, into_string},
    },
    web::auth,
};

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

// Core query logic extracted from the previous inline endpoint version.
async fn get_item(db: &Surreal<Db>, id: String, user: Option<String>) -> Result<FullWorkshopItem> {
    let id = RecordId::from_table_key("workshop_items", &id);
    let mut response = db
        .query(
            "SELECT in.appid as appid, in.description as description, in.id as id, in.title
             as title, in.author as author, in.languages as languages, in.last_updated as
             last_updated, in.score as score, in.tags.{id: id.to_string(), app_id, display_name} \
             as tags, in.preview_url as preview_url, [] as properties FROM \
             $id<-item_dependencies.*;",
        )
        .query(
            "SELECT out.appid as appid, out.description as description, out.id as id,
             out.author as author, out.languages as languages, out.last_updated as
             last_updated, out.title as title, out.score as score, out.tags.{id: id.to_string(), \
             app_id, display_name} as tags, out.preview_url as preview_url, [] as properties
             FROM $id->item_dependencies.*;",
        )
        .bind(("id", id.clone()))
        .await
        .map_err(|_| InnerError::InternalError)?;

    let dependants: Vec<WorkshopItem<RecordId>> =
        response.take(0).map_err(|_| InnerError::InternalError)?;
    let dependencies: Vec<WorkshopItem<RecordId>> =
        response.take(1).map_err(|_| InnerError::InternalError)?;

    let result = {
        let mut res = match user {
            None => db
                .query(
                    r"SELECT *, tags.{id: id.to_string(), app_id, display_name} as tags,
                ->workshop_item_properties.filter(|$prop|$prop.status == 1)[*].{
                    id: id.to_string(),
                    in: in.to_string(),
                    out: out.id.{
                        class,
                        `value`
                    },
                    source: 'system',
                    status,
                    upvote_count,
                    vote_count
                } as properties FROM $id",
                )
                .bind(("id", id.clone()))
                .await
                .map_err(|_| InnerError::InternalError)?,
            Some(user) => db
                .query(format!(
                    "SELECT *, tags.{{id: id.to_string(), app_id, display_name}} as tags,
                                    ->workshop_item_properties.filter(|$prop|$prop.status == 1 || \
                     $prop.source == {})[*].{{
                                        id: id.to_string(),
                                        in: in.to_string(),
                                        out: out.id.{{
                                            class,
                                            `value`
                                        }},
                                        source: 'system',
                                        status,
                                        upvote_count,
                                        vote_count,
                                        vote_state: votes:{{item: $id, link: out, user: {0}}}.score
                                    }} as properties FROM $id",
                    UserID::from(user).into_recordid()
                ))
                .bind(("id", id))
                .await
                .map_err(|_| InnerError::InternalError)?,
        };

        let result: Option<WorkshopItem<RecordId>> =
            res.take(0).map_err(|_| InnerError::InternalError)?;
        result.ok_or(InnerError::NotFound)?
    };

    Ok(FullWorkshopItem {
        appid: result.appid,
        description: result.description,
        id: into_string(result.id.key()),
        title: result.title,
        preview_url: result.preview_url,
        languages: result.languages,
        author: result.author,
        last_updated: result.last_updated,
        dependencies: dependencies
            .into_iter()
            .map(|e| FullWorkshopItem {
                appid: e.appid,
                author: e.author,
                dependants: vec![],
                dependencies: vec![],
                description: e.description,
                id: into_string(e.id.key()),
                languages: e.languages,
                title: e.title,
                preview_url: e.preview_url,
                last_updated: e.last_updated,
                tags: e.tags,
                score: e.score,
                properties: e.properties,
            })
            .collect(),

        dependants: dependants
            .into_iter()
            .map(|e| FullWorkshopItem {
                appid: e.appid,
                author: e.author,
                dependants: vec![],
                dependencies: vec![],
                description: e.description,
                id: into_string(e.id.key()),
                languages: e.languages,
                title: e.title,
                preview_url: e.preview_url,
                last_updated: e.last_updated,
                tags: e.tags,
                score: e.score,
                properties: e.properties,
            })
            .collect(),
        tags: result.tags,
        score: result.score,
        properties: result.properties,
    })
}

/// GET /api/item/{id}
/// Retrieves a full workshop item by id, including dependencies and dependants.
#[endpoint]
#[instrument]
pub async fn get(id: PathParam<String>, depot: &mut Depot) -> Result<Json<FullWorkshopItem>> {
    // Lazily spawn the actor on first use and keep a global reference like auth.rs
    let actor = ITEM_ACTOR.get().cloned().ok_or(InnerError::InternalError)?;

    let user = auth::get_user_from_depot(depot);
    let data = call!(actor, |reply| { ItemMsg::Get(id.0, user, reply) })
        .map_err(|_| InnerError::InternalError)??;
    Ok(Json(data))
}
