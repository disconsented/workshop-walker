use std::{collections::HashSet, sync::Arc, time::Duration};

use itertools::Itertools;
use ractor::{Actor, ActorProcessingErr, ActorRef, async_trait};
use reqwest::{Client, Response, StatusCode};
use surrealdb::{RecordId, Surreal, engine::local::Db};
use tokio::{sync::mpsc, task::JoinHandle, time::sleep};
use tracing::{debug, error};

use crate::{
    application::user_names_service::UserNamesService,
    db::user_names_repository::UserNamesSilo,
    steam::model::{SteamRoot, SteamUserResponse},
};

pub struct SteamUserActor;

pub struct SteamUserArgs {
    pub steam_token: Arc<String>,
    pub database: Surreal<Db>,
    pub client: Client,
}

pub struct SteamUserState {
    pub sender: mpsc::Sender<u64>,
    pub handle: JoinHandle<()>,
}

impl Drop for SteamUserState {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

pub enum SteamUserMsg {
    Fetch(u64),
}

#[async_trait]
impl Actor for SteamUserActor {
    type Arguments = SteamUserArgs;
    type Msg = SteamUserMsg;
    type State = SteamUserState;

    async fn pre_start(
        &self,
        _: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let (tx, mut rx) = mpsc::channel(100);

        let user_names_service = UserNamesService::new(UserNamesSilo::new(args.database));

        // Ractor doesn't really give us a good way to model this kind of work, hence
        // why this is a task instead. It's desirable to batch this work here
        // because we're trying to minimise calls to Steams API.
        let handle = tokio::spawn(async move {
            let mut total_processed = 0;
            let mut user_ids = HashSet::with_capacity(200);
            loop {
                // Process the first message to ensure that we sleep the task until there is
                // work
                let Some(first) = rx.recv().await else { break };

                if should_update_user(&user_names_service, first).await {
                    user_ids.insert(first);
                }

                while let Ok(next) = rx.try_recv() {
                    if should_update_user(&user_names_service, next).await {
                        user_ids.insert(next);
                    }
                    if user_ids.len() == 100 {
                        break;
                    }
                }

                if user_ids.is_empty() {
                    continue;
                }

                let id_string = user_ids.iter().join(",");
                let url = format!(
                    "https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/?key={}&steamids={id_string}",
                    args.steam_token
                );

                debug!(%total_processed, next_batch = user_ids.len(), "Fetching player summaries from Steam");
                // Retry up to 3 times
                for _ in 0..3 {
                    match args
                        .client
                        .get(&url)
                        .send()
                        .await
                        .and_then(Response::error_for_status)
                    {
                        Ok(resp) => match resp.json::<SteamRoot<SteamUserResponse>>().await {
                            Ok(root) => {
                                for users in root.response.players {
                                    if let Err(error) = user_names_service
                                        .update_user_name(
                                            RecordId::from_table_key("usernames", users.steamid),
                                            users.personaname,
                                        )
                                        .await
                                    {
                                        error!(?error, "Failed to update user name");
                                        break;
                                    }
                                }
                            }
                            Err(error) => {
                                error!(?error, "Failed to deserialize SteamUserResponse");
                                break;
                            }
                        },
                        Err(error) => {
                            // Back off and retry
                            if error.status() == Some(StatusCode::TOO_MANY_REQUESTS) {
                                sleep(Duration::from_secs(10)).await;
                                continue;
                            }
                            error!(%error, "Failed to fetch player summaries from Steam");
                        }
                    }
                }
                total_processed += user_ids.len();
                user_ids.clear();
            }
        });

        Ok(SteamUserState { sender: tx, handle })
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            SteamUserMsg::Fetch(id) => {
                if let Err(error) = state.sender.send(id).await {
                    error!(?error, "Failed to send steam ID to worker task");
                    panic!("Failed to send steam ID to worker task");
                }
            }
        }
        Ok(())
    }
}

async fn should_update_user(user_names_service: &UserNamesService<UserNamesSilo>, id: u64) -> bool {
    user_names_service
        .should_update_user(RecordId::from_table_key("usernames", id.to_string()))
        .await
        .unwrap_or(true)
}
