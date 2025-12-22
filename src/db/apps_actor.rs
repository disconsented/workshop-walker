use std::sync::OnceLock;

use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort, async_trait};
use surrealdb::{Surreal, engine::local::Db};

use crate::{
    application::apps_service::AppsService,
    db::{apps_repository::AppsSilo, model::App},
    domain::apps::AppError,
    steam::steam_download_actor::SteamDownloadMsg,
};

pub static APPS_ACTOR: OnceLock<ActorRef<AppsMsg>> = OnceLock::new();

pub struct AppsActor;

pub struct AppsArgs {
    pub database: Surreal<Db>,
    pub download_actor: Option<ActorRef<SteamDownloadMsg>>,
}

pub struct AppsState {
    service: AppsService<AppsSilo>,
    download_actor: Option<ActorRef<SteamDownloadMsg>>,
}

pub enum AppsMsg {
    ListAvailable(RpcReplyPort<Result<Vec<App>, AppError>>),
    Upsert(App, RpcReplyPort<Result<(), AppError>>),
    Remove(u32, RpcReplyPort<Result<(), AppError>>),
    List(RpcReplyPort<Result<Vec<App>, AppError>>),
    Get(u32, RpcReplyPort<Result<App, AppError>>),
}

#[async_trait]
impl Actor for AppsActor {
    type Arguments = AppsArgs;
    type Msg = AppsMsg;
    type State = AppsState;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        APPS_ACTOR.get_or_init(|| myself);
        Ok(AppsState {
            service: AppsService::new(AppsSilo::new(args.database)),
            download_actor: args.download_actor,
        })
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            AppsMsg::ListAvailable(reply) => {
                let _ = reply.send(state.service.list_available().await);
            }
            AppsMsg::Upsert(app, reply) => {
                let app_id = app.id;
                let res = state.service.upsert(app).await;
                if let Some(download_actor) = &state.download_actor {
                    let _ = download_actor.send_message(SteamDownloadMsg::AddApp(app_id));
                }
                let _ = reply.send(res);
            }
            AppsMsg::Remove(id, reply) => {
                let res = state.service.remove(id).await;
                if res.is_ok()
                    && let Some(download_actor) = &state.download_actor
                {
                    let _ = download_actor.send_message(SteamDownloadMsg::RemoveApp(id));
                }
                let _ = reply.send(res);
            }
            AppsMsg::List(reply) => {
                let _ = reply.send(state.service.list().await);
            }
            AppsMsg::Get(id, reply) => {
                let _ = reply.send(state.service.get(id).await);
            }
        }
        Ok(())
    }
}
