use std::{collections::HashSet, sync::Arc, time::Duration};

use itertools::Itertools;
use ractor::{Actor, ActorProcessingErr, ActorRef, async_trait};
use reqwest::{Client, Response, StatusCode};
use surrealdb::{Surreal, engine::local::Db};
use tokio::{
    sync::{mpsc, mpsc::Receiver},
    task::JoinHandle,
    time::{sleep, timeout},
};
use tracing::{debug, error};

use crate::{
    application::user_names_service::UserNamesService,
    db::{IUsernameID, user_names_repository::UserNamesSilo},
    steam::model::{SteamRoot, SteamUserResponse},
};

pub struct SteamUserActor;

pub struct SteamUserArgs {
    pub steam_token: Arc<String>,
    pub database: Surreal<Db>,
    pub client: Client,
}

pub struct SteamUserState {
    pub sender: mpsc::Sender<IUsernameID>,
    pub handle: JoinHandle<()>,
}

impl Drop for SteamUserState {
    #[tracing::instrument(level = "trace", skip(self))]
    fn drop(&mut self) {
        self.handle.abort();
    }
}

pub enum SteamUserMsg {
    Fetch(IUsernameID),
}

#[async_trait]
impl Actor for SteamUserActor {
    type Arguments = SteamUserArgs;
    type Msg = SteamUserMsg;
    type State = SteamUserState;

    #[tracing::instrument(level = "debug", skip_all)]
    async fn pre_start(
        &self,
        _: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let (tx, rx) = mpsc::channel(100);

        let user_names_service = UserNamesService::new(UserNamesSilo::new(args.database.clone()));

        // Ractor doesn't really give us a good way to model this kind of work,
        // hence why this is a task instead. It's desirable to batch
        // this work here because we're trying to minimise calls to
        // Steams API.
        let handle = tokio::spawn(Self::run_batched(args, rx, user_names_service));

        Ok(SteamUserState { sender: tx, handle })
    }

    #[tracing::instrument(level = "trace", skip(self, message, state))]
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

impl SteamUserActor {
    #[tracing::instrument(level = "trace", skip(self, args))]
    async fn run_batched(
        args: SteamUserArgs,
        mut rx: Receiver<IUsernameID>,
        user_names_service: UserNamesService<UserNamesSilo>,
    ) {
        let mut total_processed = 0;
        let mut cached_usernames = 0_usize;
        let mut user_ids: HashSet<IUsernameID> = HashSet::with_capacity(200);
        loop {
            // Process the first message to ensure that we sleep the task
            // until there is work
            let Some(first): Option<IUsernameID> = rx.recv().await else {
                break;
            };

            if should_update_user(&user_names_service, first.clone()).await {
                user_ids.insert(first);
            } else {
                cached_usernames += 1;
            }

            // Previously, we'd often get single username batches, giving it a
            // second to catch up. Squash the timeouts because I really don't
            // care, if we get none we'll just wait for the first one above.
            while let Some(next) = timeout(Duration::from_secs(1), rx.recv())
                .await
                .ok()
                .flatten()
            {
                if should_update_user(&user_names_service, next.clone()).await {
                    user_ids.insert(next);
                } else {
                    cached_usernames += 1;
                }
                if user_ids.len() == 100 {
                    break;
                }
            }

            if user_ids.is_empty() {
                continue;
            }

            let batch_size = user_ids.len();
            let id_string = user_ids
                .drain()
                .map(|id| i64::from(id.try_into_external().unwrap()).to_string())
                .join(",");
            let url = format!(
                "https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/?key={}&steamids={id_string}",
                args.steam_token
            );

            debug!(%total_processed, batch_size, cached_usernames, "Fetching player summaries from Steam");
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
                                if let Ok(id) = users.steamid.parse::<i64>()
                                    && let Err(error) = user_names_service
                                        .update_user_name(IUsernameID::from(id), users.personaname)
                                        .await
                                {
                                    error!(?error, "Failed to update user name");
                                    break;
                                }
                            }
                            break;
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
            total_processed += batch_size;
        }
    }
}

#[tracing::instrument(level = "trace", skip(user_names_service, id))]
async fn should_update_user(
    user_names_service: &UserNamesService<UserNamesSilo>,
    id: IUsernameID,
) -> bool {
    user_names_service
        .should_update_user(id)
        .await
        .inspect_err(|error| error!(?error, "Failed to check if user should be updated"))
        .unwrap_or(true)
}
