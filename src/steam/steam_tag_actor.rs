use std::time::Duration;

use ractor::{Actor, ActorProcessingErr, ActorRef, async_trait};
use reqwest::Client;
use scraper::{Html, Selector};
use surrealdb::{Surreal, engine::local::Db};
use tracing::{debug, error, info, warn};

use crate::{
    application::{apps_service::AppsService, tags_service::TagsService},
    db::{ITagID, apps_repository::AppsSilo, model::InternalTag, tags_repository::TagsSilo},
};

pub struct SteamTagActor;

pub struct SteamTagArgs {
    pub database: Surreal<Db>,
    pub client: Client,
    pub selector: String,
}

pub struct SteamTagState {
    client: Client,
    apps: AppsService<AppsSilo>,
    tags: TagsService<TagsSilo>,
    selector: Selector,
}

pub enum SteamTagMsg {
    Update,
}

#[async_trait]
impl Actor for SteamTagActor {
    type Arguments = SteamTagArgs;
    type Msg = SteamTagMsg;
    type State = SteamTagState;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        // Trigger initial update
        myself.cast(SteamTagMsg::Update)?;
        myself.send_interval(Duration::from_hours(24), || SteamTagMsg::Update);

        let selector = match Selector::parse(&args.selector) {
            Ok(selector) => selector,
            Err(error) => {
                error!(?error, "parsing selector");
                panic!("parsing selector"); // give up on the error will fix later
            }
        };

        Ok(SteamTagState {
            client: args.client,
            apps: AppsService::new(AppsSilo::new(args.database.clone())),
            tags: TagsService::new(TagsSilo::new(args.database)),
            selector,
        })
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            SteamTagMsg::Update => {
                let apps = match state.apps.list_available().await {
                    Ok(apps) => apps,
                    Err(error) => {
                        error!(?error, "Failed to list available apps");
                        return Ok(());
                    }
                };

                for app in apps {
                    info!(app = ?app.id, app_name = %app.name, "Scraping tags for app");
                    let url = format!(
                        "https://steamcommunity.com/app/{}/workshop/",
                        i64::from(app.clone().id.clone().try_into_external()?)
                    );
                    match state.client.get(&url).send().await {
                        Ok(resp) => {
                            if let Ok(html) = resp.text().await {
                                let tags = extract_tags(&state.selector, &html);

                                if tags.len() == 0 {
                                    warn!(app = ?app.id, "No tags found for app")
                                } else {
                                    debug!(app = ?app.id, tag_count = tags.len(), "Extracted tags");
                                }

                                if let Err(error) =
                                    state.tags.update_tags(app.id.clone(), tags).await
                                {
                                    error!(?error, app = ?app.id, "Failed to update tags");
                                }
                            }
                        }
                        Err(error) => {
                            error!(?error, app = ?app.id, "Failed to fetch workshop page");
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/// N.B. this will probably break periodically as valve does things, need to do
/// something more permanent See https://github.com/disconsented/workshop-walker/issues/49
fn extract_tags(selector: &Selector, html: &str) -> Vec<InternalTag> {
    Html::parse_document(html)
        .select(&selector)
        .filter_map(|node| {
            node.text()
                .collect::<String>()
                .split("\u{a0}")
                .next()
                .map(String::from)
        })
        .map(|text| InternalTag {
            id: ITagID::from(text.clone()),
            display_name: text.clone(),
        })
        .collect::<Vec<_>>()
}
#[cfg(test)]
mod tests {
    use scraper::{Html, Selector};

    #[test]
    fn test_extract_tags() {
        let html = r#"
<div class="_3WoKyweAE-g-"><div class="_8dYm3fztICg- Panel" tabindex="0" role="button"><span class="KX9eQJSfx5A- NI9oaXH36YQ- gGpgfDgWbuw-">Mod</span><span class="NI9oaXH36YQ- xoIecxRL27Q-">(35,768)</span></div><div class="_8dYm3fztICg- Panel" tabindex="0" role="button"><span class="KX9eQJSfx5A- NI9oaXH36YQ- gGpgfDgWbuw-">Translation</span><span class="NI9oaXH36YQ- xoIecxRL27Q-">(12,409)</span></div><div class="_8dYm3fztICg- Panel" tabindex="0" role="button"><span class="KX9eQJSfx5A- NI9oaXH36YQ- gGpgfDgWbuw-">Scenario</span><span class="NI9oaXH36YQ- xoIecxRL27Q-">(7,621)</span></div><div class="_8dYm3fztICg- Panel" tabindex="0" role="button"><span class="KX9eQJSfx5A- NI9oaXH36YQ- gGpgfDgWbuw-">0.14</span><span class="NI9oaXH36YQ- xoIecxRL27Q-">(909)</span></div><div class="_8dYm3fztICg- Panel" tabindex="0" role="button"><span class="KX9eQJSfx5A- NI9oaXH36YQ- gGpgfDgWbuw-">0.15</span><span class="NI9oaXH36YQ- xoIecxRL27Q-">(558)</span></div><div class="_8dYm3fztICg- Panel" tabindex="0" role="button"><span class="KX9eQJSfx5A- NI9oaXH36YQ- gGpgfDgWbuw-">0.16</span><span class="NI9oaXH36YQ- xoIecxRL27Q-">(913)</span></div><div class="_8dYm3fztICg- Panel" tabindex="0" role="button"><span class="KX9eQJSfx5A- NI9oaXH36YQ- gGpgfDgWbuw-">0.17</span><span class="NI9oaXH36YQ- xoIecxRL27Q-">(1,094)</span></div><div class="_8dYm3fztICg- Panel" tabindex="0" role="button"><span class="KX9eQJSfx5A- NI9oaXH36YQ- gGpgfDgWbuw-">0.18</span><span class="NI9oaXH36YQ- xoIecxRL27Q-">(1,562)</span></div><div class="_8dYm3fztICg- Panel" tabindex="0" role="button"><span class="KX9eQJSfx5A- NI9oaXH36YQ- gGpgfDgWbuw-">0.19</span><span class="NI9oaXH36YQ- xoIecxRL27Q-">(544)</span></div><div class="_8dYm3fztICg- Panel" tabindex="0" role="button"><span class="KX9eQJSfx5A- NI9oaXH36YQ- gGpgfDgWbuw-">1.0</span><span class="NI9oaXH36YQ- xoIecxRL27Q-">(7,826)</span></div><div class="_8dYm3fztICg- Panel" tabindex="0" role="button"><span class="KX9eQJSfx5A- NI9oaXH36YQ- gGpgfDgWbuw-">1.1</span><span class="NI9oaXH36YQ- xoIecxRL27Q-">(8,776)</span></div><div class="_8dYm3fztICg- Panel" tabindex="0" role="button"><span class="KX9eQJSfx5A- NI9oaXH36YQ- gGpgfDgWbuw-">1.2</span><span class="NI9oaXH36YQ- xoIecxRL27Q-">(11,280)</span></div><div class="_8dYm3fztICg- Panel" tabindex="0" role="button"><span class="KX9eQJSfx5A- NI9oaXH36YQ- gGpgfDgWbuw-">1.3</span><span class="NI9oaXH36YQ- xoIecxRL27Q-">(16,152)</span></div><div class="_8dYm3fztICg- Panel" tabindex="0" role="button"><span class="KX9eQJSfx5A- NI9oaXH36YQ- gGpgfDgWbuw-">1.4</span><span class="NI9oaXH36YQ- xoIecxRL27Q-">(21,051)</span></div><div class="_8dYm3fztICg- Panel" tabindex="0" role="button"><span class="KX9eQJSfx5A- NI9oaXH36YQ- gGpgfDgWbuw-">1.5</span><span class="NI9oaXH36YQ- xoIecxRL27Q-">(23,571)</span></div><div class="_8dYm3fztICg- Panel" tabindex="0" role="button"><span class="KX9eQJSfx5A- NI9oaXH36YQ- gGpgfDgWbuw-">1.6</span><span class="NI9oaXH36YQ- xoIecxRL27Q-">(27,405)</span></div></div>
        "#;
        let document = Html::parse_document(html);
        let tags = document
            .select(&Selector::parse("div._8dYm3fztICg- > span:nth-child(1)").unwrap())
            .map(|node| {
                node.text()
                    .collect::<String>()
                    .split("\u{a0}")
                    .next()
                    .unwrap()
                    .to_string()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            tags,
            vec![
                "Mod",
                "Translation",
                "Scenario",
                "0.14",
                "0.15",
                "0.16",
                "0.17",
                "0.18",
                "0.19",
                "1.0",
                "1.1",
                "1.2",
                "1.3",
                "1.4",
                "1.5",
                "1.6"
            ]
        );
    }
}
