use classification::actor::{ExtractionActor, ExtractionArgs};
use ractor::Actor;
use reqwest::Client;
use snafu::{ResultExt, Whatever};
use surrealdb::{Surreal, engine::local::Db};
use tracing::{Instrument, info_span, instrument};

use crate::{
    app_config::Config,
    db::{
        item_update_actor::{ItemUpdateActor, ItemUpdateArgs},
        properties_actor::{PropertiesActor, PropertiesArgs},
    },
    processing::{
        bb_actor::{BBActor, BBArgs},
        language_actor::{LanguageActor, LanguageArgs},
        ml_queue_actor::{MLQueueActor, MLQueueArgs},
    },
    steam::steam_download_actor::{SteamDownloadActor, SteamDownloadArgs},
    web::{
        auth::{AuthActor, AuthArgs},
        item::{ItemActor, ItemArgs},
    },
};

#[instrument(skip_all)]
pub async fn spawn(config: &Config, db: &Surreal<Db>) -> Result<(), Whatever> {
    let reqwest_client = Client::new();

    let (language_actor, _) = Actor::spawn(
        Some("/language".to_string()),
        LanguageActor {},
        LanguageArgs {},
    )
    .instrument(info_span!("spawn::language"))
    .await
    .whatever_context("Spawning language actor")?;
    let (bb_actor, _) = Actor::spawn(Some("/bb".to_string()), BBActor {}, BBArgs {})
        .instrument(info_span!("spawn::language"))
        .await
        .whatever_context("Spawning bb actor")?;

    let (extraction_actor, _) = Actor::spawn(
        Some("/ml_extractor".to_string()),
        ExtractionActor,
        ExtractionArgs {},
    )
    .instrument(info_span!("spawn::extraction"))
    .await
    .whatever_context("Spawning ML extraction actor")?;

    let (property_actor, _) = Actor::spawn(
        Some("/properties".to_string()),
        PropertiesActor,
        PropertiesArgs {
            database: db.clone(),
        },
    )
    .instrument(info_span!("spawn::properties"))
    .await
    .whatever_context("Spawning properties actor")?;

    let (ml_queue_actor, _) = Actor::spawn(
        Some("/ml_queue".to_string()),
        MLQueueActor,
        MLQueueArgs {
            database: db.clone(),
            extractor: extraction_actor,
            property_actor,
        },
    )
    .instrument(info_span!("spawn::ml_queue"))
    .await
    .whatever_context("Spawning ML queue actor")?;
    let (item_update_actor, _) = Actor::spawn(
        Some("/item_updater".to_string()),
        ItemUpdateActor {},
        ItemUpdateArgs {
            language_actor,
            bb_actor,
            database: db.clone(),
            ml_queue: config.ml_extraction.then_some(ml_queue_actor),
        },
    )
    .instrument(info_span!("spawn::item_update"))
    .await
    .whatever_context("Spawning item_update actor")?;
    let (..) = Actor::spawn(
        Some("/auth".to_string()),
        AuthActor {},
        AuthArgs {
            database: db.clone(),
            client: reqwest_client.clone(),
            base_url: config.base_url.clone(),
            biscuit: config.biscuit.clone(),
        },
    )
    .instrument(info_span!("spawn::auth"))
    .await
    .whatever_context("Spawning auth actor")?;
    if config.updater {
        let (..) = Actor::spawn(
            Some("/steam-download".to_string()),
            SteamDownloadActor {},
            SteamDownloadArgs {
                steam_token: config.steam.api_token.clone(),
                item_processing_actor_ref: item_update_actor,
                database: db.clone(),
                app_id: config.steam.appid,
                client: reqwest_client,
                force: config.force_update,
            },
        )
        .instrument(info_span!("spawn::steam_download"))
        .await
        .whatever_context("Spawning steam download actor")?;
    }

    let (..) = Actor::spawn(
        Some("/item".to_string()),
        ItemActor,
        ItemArgs {
            database: db.clone(),
        },
    )
    .instrument(info_span!("spawn::item"))
    .await
    .whatever_context("Spawning item actor")?;

    Ok(())
}
