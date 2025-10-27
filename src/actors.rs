use classification::actor::{ExtractionActor, ExtractionArgs};
use ractor::Actor;
use reqwest::Client;
use snafu::{ResultExt, Whatever};
use surrealdb::{engine::local::Db, Surreal};

use crate::{
    app_config::Config,
    db::item_update_actor::{ItemUpdateActor, ItemUpdateArgs},
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
use crate::db::properties_actor::{PropertiesActor, PropertiesArgs};

pub async fn spawn(config: &Config, db: &Surreal<Db>) -> Result<(), Whatever> {
    let reqwest_client = Client::new();

    let (language_actor, _) = Actor::spawn(
        Some("/language".to_string()),
        LanguageActor {},
        LanguageArgs {},
    )
    .await
    .whatever_context("Spawning language actor")?;
    let (bb_actor, _) = Actor::spawn(Some("/bb".to_string()), BBActor {}, BBArgs {})
        .await
        .whatever_context("Spawning bb actor")?;

    let (extraction_actor, _) = Actor::spawn(
        Some("/ml_extractor".to_string()),
        ExtractionActor,
        ExtractionArgs {},
    )
    .await
    .whatever_context("Spawning ML extraction actor")?;

    let (property_actor, _) = Actor::spawn(
        Some("/properties".to_string()),
        PropertiesActor,
        PropertiesArgs {
            database: db.clone(),
        },
    )
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
            },
        )
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
    .await
    .whatever_context("Spawning item actor")?;

    Ok(())
}
