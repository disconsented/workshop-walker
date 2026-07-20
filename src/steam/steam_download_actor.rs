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
    db::{IAppID, item_update_actor::ItemUpdateMsg},
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
    apps: HashMap<IAppID, JoinHandle<()>>,
    database: Surreal<Db>,
}

pub enum SteamDownloadMsg {
    Download { app: IAppID, first_page: GetPage },
    AddApp(IAppID),
    RemoveApp(IAppID),
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
        let apps: Vec<IAppID> = args
            .database
            .query("SELECT id AS id FROM apps WHERE enabled = true")
            .await?
            .take((0, "id"))?;

        let mut state = Self::State {
            client: args.client,
            steam_token: args.steam_token,
            item_processing_actor_ref: args.item_processing_actor_ref,
            apps: HashMap::new(),
            database: args.database,
        };
        for app in apps {
            start_downloader(&myself, &mut state, app.clone(), args.force)
                .instrument(info_span!("start downloader", ?app))
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
            SteamDownloadMsg::Download { app, first_page } => {
                if let Err(error) = download(
                    state,
                    app.clone(),
                    first_page,
                    state.item_processing_actor_ref.clone(),
                )
                .await
                {
                    error!(?app, ?error, "Downloading workshop items");
                }
            }
            SteamDownloadMsg::AddApp(app) => {
                if !state.apps.contains_key(&app) {
                    start_downloader(&myself, state, app, false).await;
                }
            }
            SteamDownloadMsg::RemoveApp(app) => {
                if let Some(handle) = state.apps.remove(&app) {
                    handle.abort();
                    info!(?app, "Stopped downloading workshop items");
                }
            }
        }

        Ok(())
    }
}

async fn download(
    state: &mut SteamDownloadState,
    app: IAppID,
    mut page: GetPage,
    database_writer_actor_ref: ActorRef<ItemUpdateMsg>,
) -> Result<(), Whatever> {
    let app = app
        .try_into_external()
        .whatever_context("converting app id")?
        .into();
    page.appid = app;
    let mut total = i64::MAX;
    let mut downloaded = 0;
    while total > downloaded {
        page.appid = app;
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
            ?app,
            "Downloaded items"
        );
    }
    Ok(())
}

async fn start_downloader(
    myself: &ActorRef<SteamDownloadMsg>,
    state: &mut SteamDownloadState,
    app: IAppID, // Function needs to be infalliable, so, we handle the converion outside here
    force: bool,
) {
    let timestamp: Option<u64> = state
        .database
        .query(
            "SELECT last_updated FROM workshop_items WHERE app = $app ORDER BY last_updated \
             DESC LIMIT 1",
        )
        .bind(("app", app.clone()))
        .await
        .unwrap()
        .take((0, "last_updated"))
        .unwrap();
    let timestamp = timestamp.unwrap_or(0);
    let time_since = SystemTime::now()
        .duration_since(UNIX_EPOCH.add(Duration::from_secs(timestamp)))
        .unwrap();
    let h12 = Duration::from_hours(12);

    let message_builder = {
        let app = app.clone();
        move || {
            let app = app.clone();
            SteamDownloadMsg::Download {
                app,
                first_page: GetPage {
                    query_type: EPublishedFileQueryType::RankedByLastUpdatedDate,
                    ..Default::default()
                },
            }
        }
    };
    if time_since > h12 || force {
        let _ = myself.send_message(message_builder());
        info!(period = %humantime::Duration::from(time_since), app = ?app, "newest mod is at least 12 hours out of date; running update now");
    }

    if let Some(old) = state
        .apps
        .insert(app.clone(), myself.send_interval(h12, message_builder))
    {
        // Remember to abort the old timer
        old.abort();
    }
}
