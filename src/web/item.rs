use std::sync::OnceLock;

use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort, async_trait, call};
use salvo::{
    Depot, Writer,
    oapi::{endpoint, extract::PathParam},
    prelude::{Json, StatusCode, StatusError},
};
use surrealdb::{Surreal, engine::local::Db};
use tracing::{error, instrument};

use crate::{
    application::items_service::ItemsService,
    db::{
        IItemID, IUserID,
        items_repository::ItemsSilo,
        model::{ExternalFullWorkshopItem, InternalFullWorkshopItem},
    },
    processing::ml_activity_actor::MLActivityMsg,
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
    items_service: ItemsService<ItemsSilo>,
    ml_activity_actor: ActorRef<MLActivityMsg>,
}
pub struct ItemArgs {
    pub database: Surreal<Db>,
    pub ml_activity_actor: ActorRef<MLActivityMsg>,
}

pub enum ItemMsg {
    Get(
        IItemID,
        Option<IUserID>,
        RpcReplyPort<Result<InternalFullWorkshopItem>>,
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
            items_service: ItemsService::new(ItemsSilo::new(args.database)),
            ml_activity_actor: args.ml_activity_actor,
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
                let res = state
                    .items_service
                    .get_full_item(id.clone(), user)
                    .await
                    .map_err(Into::into);

                // If we get this far the item exists
                if res.is_ok() {
                    let _ = state
                        .ml_activity_actor
                        .send_message(MLActivityMsg::Signal(id));
                }

                if reply.send(res).is_err() {
                    error!(message = "Get", "Failed to reply to message");
                }
            }
        }
        Ok(())
    }
}

/// GET /api/item/{id}
/// Retrieves a full workshop item by id, including dependencies and dependants.
#[endpoint]
#[instrument(skip_all)]
pub async fn get(id: PathParam<i64>, depot: &mut Depot) -> Result<Json<ExternalFullWorkshopItem>> {
    // ToDo: Use the service directly at some point rather than an actor
    // Lazily spawn the actor on first use and keep a global reference like
    // auth.rs
    let item_actor = ITEM_ACTOR.get().cloned().ok_or(InnerError::InternalError)?;

    let user = auth::get_user_from_depot(depot);
    let data = call!(item_actor, |reply| {
        ItemMsg::Get(id.0.into(), user, reply)
    })
    .map_err(|_| InnerError::InternalError)??;
    Ok(Json(
        data.try_into().map_err(|_| InnerError::InternalError)?,
    ))
}
