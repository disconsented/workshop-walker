use std::sync::OnceLock;

use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort, async_trait};
use surrealdb::{Surreal, engine::local::Db};
use tracing::{Instrument, debug_span};

use crate::{
    application::apps_service::AppsService,
    db::{IAppID, apps_repository::AppsSilo, model::InternalApp},
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

/// What the actor takes.
pub type AppsMsg = AppsRequest;

pub enum AppsRequest {
    ListAvailable(RpcReplyPort<Result<Vec<InternalApp>, AppError>>),
    Upsert(InternalApp, RpcReplyPort<Result<(), AppError>>),
    Remove(IAppID, RpcReplyPort<Result<(), AppError>>),
    List(RpcReplyPort<Result<Vec<InternalApp>, AppError>>),
    Get(IAppID, RpcReplyPort<Result<InternalApp, AppError>>),
}

#[async_trait]
impl Actor for AppsActor {
    type Arguments = AppsArgs;
    type Msg = AppsMsg;
    type State = AppsState;

    #[tracing::instrument(level = "debug", skip_all)]
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
            AppsRequest::ListAvailable(reply) => {
                let _ = reply.send(
                    state
                        .service
                        .list_available()
                        .instrument(debug_span!("apps list available"))
                        .await,
                );
            }
            AppsRequest::Upsert(app, reply) => {
                let app_id = app.id.clone();
                let res = state
                    .service
                    .upsert(app)
                    .instrument(debug_span!("apps upsert", app.id = ?app_id.key))
                    .await;
                if let Some(download_actor) = &state.download_actor {
                    let _ = download_actor.send_message(SteamDownloadMsg::AddApp(app_id));
                }
                let _ = reply.send(res);
            }
            AppsRequest::Remove(id, reply) => {
                let res = state
                    .service
                    .remove(id.clone())
                    .instrument(debug_span!("apps remove", app.id = ?id.key))
                    .await;
                if res.is_ok()
                    && let Some(download_actor) = &state.download_actor
                {
                    let _ = download_actor.send_message(SteamDownloadMsg::RemoveApp(id));
                }
                let _ = reply.send(res);
            }
            AppsRequest::List(reply) => {
                let _ = reply.send(
                    state
                        .service
                        .list()
                        .instrument(debug_span!("apps list"))
                        .await,
                );
            }
            AppsRequest::Get(id, reply) => {
                let span = debug_span!("apps get", app.id = ?id.key);
                let _ = reply.send(state.service.get(id).instrument(span).await);
            }
        }
        Ok(())
    }
}
