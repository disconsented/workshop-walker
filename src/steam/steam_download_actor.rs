use std::{
    ops::Add,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use ractor::{Actor, ActorProcessingErr, ActorRef, async_trait};
use reqwest::Client;
use snafu::{ResultExt, Whatever};
use surrealdb::{Surreal, engine::local::Db};
use tracing::{debug, error, info};

use crate::{
    db::item_update_actor::ItemUpdateMsg,
    steam::model::{EPublishedFileQueryType, GetPage, IPublishedResponse, SteamRoot},
};

pub struct SteamDownloadActor {}

pub struct SteamDownloadArgs {
    pub steam_token: Arc<String>,
    pub item_processing_actor_ref: ActorRef<ItemUpdateMsg>,
    pub database: Surreal<Db>,
    pub app_id: u32,
    pub client: Client,
}
pub struct SteamDownloadState {
    client: Client,
    steam_token: Arc<String>,
    item_processing_actor_ref: ActorRef<ItemUpdateMsg>,
}

pub enum SteamDownloadMsg {
    Download { app_id: u32, first_page: GetPage },
}
#[async_trait]
impl Actor for SteamDownloadActor {
    type Arguments = SteamDownloadArgs;
    type Msg = SteamDownloadMsg;
    type State = SteamDownloadState;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        // ToDo: do this per app
        let timestamp: Option<u64> = args
            .database
            .query("SELECT last_updated FROM workshop_items ORDER BY last_updated DESC LIMIT 1")
            .await
            .unwrap()
            .take((0, "last_updated"))
            .unwrap();
        let timestamp = timestamp.unwrap_or(0);
        let time_since = SystemTime::now()
            .duration_since(UNIX_EPOCH.add(Duration::from_secs(timestamp)))
            .unwrap();
        let h12 = Duration::from_secs(60 * 60 * 12);
        let message_builder = move || SteamDownloadMsg::Download {
            app_id: args.app_id,
            first_page: GetPage {
                query_type: EPublishedFileQueryType::RankedByLastUpdatedDate,
                ..Default::default()
            },
        };
        if time_since > h12 {
            myself.send_message(message_builder())?;
            info!("last_updated: {}", humantime::Duration::from(time_since));
        }

        myself.send_interval(h12, message_builder);
        Ok(Self::State {
            client: args.client,
            steam_token: args.steam_token,
            item_processing_actor_ref: args.item_processing_actor_ref,
        })
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            SteamDownloadMsg::Download { app_id, first_page } => {
                if let Err(e) = download(
                    state,
                    app_id,
                    first_page,
                    state.item_processing_actor_ref.clone(),
                )
                .await
                {
                    error!("Downloading workshop items for {app_id} with err: {e:?}");
                }
            }
        }

        Ok(())
    }
}

async fn download(
    state: &mut SteamDownloadState,
    app_id: u32,
    mut page: GetPage,
    database_writer_actor_ref: ActorRef<ItemUpdateMsg>,
) -> Result<(), Whatever> {
    page.appid = app_id;
    let mut total = i64::MAX;
    let mut downloaded = 0;
    while total >= downloaded {
        page.appid = app_id;
        let request = page
            .into_request(&state.client, &state.steam_token)
            .whatever_context("building download request")?;
        let response = state
            .client
            .execute(request)
            .await
            .whatever_context("Sending get page request")?;
        let json = response
            .json::<SteamRoot<IPublishedResponse>>()
            .await
            .whatever_context("request body")?;

        if json.response.publishedfiledetails.is_empty() {
            debug!("Got fewer than expected items; exiting early");
            break;
        }

        total = json.response.total;
        page = GetPage::try_from(&json)?;
        downloaded += json.response.publishedfiledetails.len() as i64;
        database_writer_actor_ref
            .send_message(ItemUpdateMsg::DeserializeRawFiles(json))
            .whatever_context("forwarding to the database actor")?;
        debug!(
            progress = (downloaded * 100 / total * 100),
            downloaded,
            expected = total,
            app_id,
            "Downloaded items"
        );
    }
    Ok(())
}
