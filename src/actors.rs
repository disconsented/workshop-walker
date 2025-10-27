use ractor::Actor;
use reqwest::Client;
use snafu::{ResultExt, Whatever};
use surrealdb::{Surreal, engine::local::Db};

use crate::{
    app_config::Config,
    db::item_update_actor::{ItemUpdateActor, ItemUpdateArgs},
    processing::{
        bb_actor::{BBActor, BBArgs},
        language_actor::{LanguageActor, LanguageArgs},
    },
    steam::steam_download_actor::{SteamDownloadActor, SteamDownloadArgs},
    web::{
        auth::{AuthActor, AuthArgs},
        item::{ItemActor, ItemArgs},
        properties::{PropertiesActor, PropertiesArgs},
    },
};

pub async fn spawn(config: &Config, db: &Surreal<Db>) -> Result<(), Whatever> {
    let client = Client::new();
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
    let (item_update_actor, _) = Actor::spawn(
        Some("/item_updater".to_string()),
        ItemUpdateActor {},
        ItemUpdateArgs {
            language_actor,
            bb_actor,
            database: db.clone(),
        },
    )
    .await
    .whatever_context("Spawning item_update actor")?;
    let (..) = Actor::spawn(
        Some("/auth".to_string()),
        AuthActor {},
        AuthArgs {
            database: db.clone(),
            client: client.clone(),
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
                client,
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

    let (..) = Actor::spawn(
        Some("/properties".to_string()),
        PropertiesActor,
        PropertiesArgs {
            database: db.clone(),
        },
    )
    .await
    .whatever_context("Spawning properties actor")?;

    Ok(())
}
