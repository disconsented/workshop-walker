use std::sync::OnceLock;

use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort, async_trait};
use surrealdb::{Surreal, engine::local::Db};

use crate::{
    application::admin_service::AdminService,
    db::{
        admin_repository::AdminSilo,
        model::{Property, User, WorkshopItemProperties},
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

pub enum AdminMsg {
    ListUsers(RpcReplyPort<Result<Vec<User<String>>, AdminError>>),
    PatchUser(PatchUserData, RpcReplyPort<Result<(), AdminError>>),
    ListWorkshopItemProperties(
        RpcReplyPort<Result<Vec<WorkshopItemProperties<String, Property>>, AdminError>>,
    ),
    PatchWorkshopItemProperty(PatchRelationshipData, RpcReplyPort<Result<(), AdminError>>),
}

#[async_trait]
impl Actor for AdminActor {
    type Arguments = AdminArgs;
    type Msg = AdminMsg;
    type State = AdminState;

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
            AdminMsg::ListUsers(reply) => {
                let res = state.service.list_users().await;
                let _ = reply.send(res);
            }
            AdminMsg::PatchUser(patch, reply) => {
                let res = state.service.patch_user(patch).await;
                let _ = reply.send(res);
            }
            AdminMsg::ListWorkshopItemProperties(reply) => {
                let res = state.service.list_workshop_item_properties().await;
                let _ = reply.send(res);
            }
            AdminMsg::PatchWorkshopItemProperty(patch, reply) => {
                let res = state.service.patch_workshop_item_property(patch).await;
                let _ = reply.send(res);
            }
        }
        Ok(())
    }
}
