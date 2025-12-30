use std::{
    collections::HashMap,
    ops::Add,
    sync::{Arc, OnceLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use ractor::{Actor, ActorProcessingErr, ActorRef, async_trait};
use reqwest::Client;
use snafu::{ResultExt, Whatever};
use surrealdb::{Surreal, engine::local::Db};
use tokio::task::JoinHandle;
use tracing::{Instrument, debug, error, info, info_span};

use crate::{
    db::item_update_actor::ItemUpdateMsg,
    steam::model::{EPublishedFileQueryType, GetPage, IPublishedResponse, SteamRoot},
};

pub static DOWNLOAD_ACTOR: OnceLock<ActorRef<SteamDownloadMsg>> = OnceLock::new();

pub struct SteamDownloadActor {}

pub struct SteamDownloadArgs {
    pub steam_token: Arc<String>,
    pub item_processing_actor_ref: ActorRef<ItemUpdateMsg>,
    pub database: Surreal<Db>,
    pub client: Client,
    pub force: bool,
}
pub struct SteamDownloadState {
    client: Client,
    steam_token: Arc<String>,
    item_processing_actor_ref: ActorRef<ItemUpdateMsg>,
    apps: HashMap<u32, JoinHandle<()>>,
    database: Surreal<Db>,
}

pub enum SteamDownloadMsg {
    Download { app_id: u32, first_page: GetPage },
    AddApp(u32),
    RemoveApp(u32),
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
        let apps: Vec<u32> = args
            .database
            .query("SELECT id.id() AS id FROM apps WHERE enabled = true")
            // .instrument(info_span!("select enabled apps"))
            .await?
            .take((0, "id"))?;

        let mut state = Self::State {
            client: args.client,
            steam_token: args.steam_token,
            item_processing_actor_ref: args.item_processing_actor_ref,
            apps: HashMap::new(),
            database: args.database,
        };
        for app_id in apps {
            start_downloader(&myself, &mut state, app_id, args.force)
                .instrument(info_span!("start downloader", app_id))
                .await;
        }

        DOWNLOAD_ACTOR.get_or_init(|| myself);
        Ok(state)
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            SteamDownloadMsg::Download { app_id, first_page } => {
                if let Err(error) = download(
                    state,
                    app_id,
                    first_page,
                    state.item_processing_actor_ref.clone(),
                )
                .await
                {
                    error!(app_id, ?error, "Downloading workshop items");
                }
            }
            SteamDownloadMsg::AddApp(app_id) => {
                if !state.apps.contains_key(&app_id) {
                    start_downloader(&myself, state, app_id, false).await;
                }
            }
            SteamDownloadMsg::RemoveApp(app_id) => {
                if let Some(handle) = state.apps.remove(&app_id) {
                    handle.abort();
                    info!(app_id, "Stopped downloading workshop items");
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
    while total > downloaded {
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
            progress = (downloaded * 100 / total * 100) / 100,
            downloaded,
            expected = total,
            app_id,
            "Downloaded items"
        );
    }
    Ok(())
}

async fn start_downloader(
    myself: &ActorRef<SteamDownloadMsg>,
    state: &mut SteamDownloadState,
    app_id: u32,
    force: bool,
) {
    let timestamp: Option<u64> = state
        .database
        .query(
            "SELECT last_updated FROM workshop_items WHERE appid = $appid ORDER BY last_updated \
             DESC LIMIT 1",
        )
        .bind(("appid", app_id))
        .await
        .unwrap()
        .take((0, "last_updated"))
        .unwrap();
    let timestamp = timestamp.unwrap_or(0);
    let time_since = SystemTime::now()
        .duration_since(UNIX_EPOCH.add(Duration::from_secs(timestamp)))
        .unwrap();
    let h12 = Duration::from_hours(12);
    let message_builder = move || SteamDownloadMsg::Download {
        app_id,
        first_page: GetPage {
            query_type: EPublishedFileQueryType::RankedByLastUpdatedDate,
            ..Default::default()
        },
    };
    if time_since > h12 || force {
        let _ = myself.send_message(message_builder());
        info!(period = %humantime::Duration::from(time_since),app = app_id, "newest mod is at least 12 hours out of date; running update now");
    }

    if let Some(old) = state
        .apps
        .insert(app_id, myself.send_interval(h12, message_builder))
    {
        // Remember to abort the old timer
        old.abort();
    }
}
