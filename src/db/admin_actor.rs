use std::sync::OnceLock;

use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort, async_trait};
use surrealdb::{Surreal, engine::local::Db};
use tracing::{Instrument, debug_span};

use crate::{
    application::admin_service::AdminService,
    db::{
        admin_repository::AdminSilo,
        model::{InternalUser, InternalWorkshopItemProperties},
    },
    domain::admin::{AdminError, PatchRelationshipData, PatchUserData},
};

pub static ADMIN_ACTOR: OnceLock<ActorRef<AdminMsg>> = OnceLock::new();

pub struct AdminActor;

pub struct AdminArgs {
    pub database: Surreal<Db>,
}

pub struct AdminState {
    service: AdminService<AdminSilo>,
}

/// What the actor takes.
pub type AdminMsg = AdminRequest;

pub enum AdminRequest {
    ListUsers(RpcReplyPort<Result<Vec<InternalUser>, AdminError>>),
    PatchUser(PatchUserData, RpcReplyPort<Result<(), AdminError>>),
    ListWorkshopItemProperties(
        RpcReplyPort<Result<Vec<InternalWorkshopItemProperties>, AdminError>>,
    ),
    PatchWorkshopItemProperty(PatchRelationshipData, RpcReplyPort<Result<(), AdminError>>),
}

#[async_trait]
impl Actor for AdminActor {
    type Arguments = AdminArgs;
    type Msg = AdminMsg;
    type State = AdminState;

    #[tracing::instrument(level = "debug", skip_all)]
    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        ADMIN_ACTOR.get_or_init(|| myself);
        Ok(AdminState {
            service: AdminService::new(AdminSilo::new(args.database)),
        })
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            AdminRequest::ListUsers(reply) => {
                let res = state
                    .service
                    .list_users()
                    .instrument(debug_span!("admin list users"))
                    .await;
                let _ = reply.send(res);
            }
            AdminRequest::PatchUser(patch, reply) => {
                let span = debug_span!("admin patch user", user.id = ?patch.id);
                let res = state.service.patch_user(patch).instrument(span).await;
                let _ = reply.send(res);
            }
            AdminRequest::ListWorkshopItemProperties(reply) => {
                let res = state
                    .service
                    .list_workshop_item_properties()
                    .instrument(debug_span!("admin list item properties"))
                    .await;
                let _ = reply.send(res);
            }
            AdminRequest::PatchWorkshopItemProperty(patch, reply) => {
                let res = state
                    .service
                    .patch_workshop_item_property(patch)
                    .instrument(debug_span!("admin patch item property"))
                    .await;
                let _ = reply.send(res);
            }
        }
        Ok(())
    }
}
