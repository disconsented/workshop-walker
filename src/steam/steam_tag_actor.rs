use std::time::Duration;

use ractor::{Actor, ActorProcessingErr, ActorRef, async_trait};
use reqwest::Client;
use scraper::{Html, Selector};
use surrealdb::{Surreal, engine::local::Db};
use tracing::{debug, error, info};

use crate::{
    application::{apps_service::AppsService, tags_service::TagsService},
    db::{apps_repository::AppsSilo, model::Tag, tags_repository::TagsSilo},
};

pub struct SteamTagActor;

pub struct SteamTagArgs {
    pub database: Surreal<Db>,
    pub client: Client,
}

pub struct SteamTagState {
    client: Client,
    apps: AppsService<AppsSilo>,
    tags: TagsService<TagsSilo>,
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

        Ok(SteamTagState {
            client: args.client,
            apps: AppsService::new(AppsSilo::new(args.database.clone())),
            tags: TagsService::new(TagsSilo::new(args.database)),
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
                    info!(app_id = app.id, app_name = %app.name, "Scraping tags for app");
                    let url = format!("https://steamcommunity.com/app/{}/workshop/", app.id);
                    match state.client.get(&url).send().await {
                        Ok(resp) => {
                            if let Ok(html) = resp.text().await {
                                let tags = extract_tags(app.id, &html);
                                debug!(app_id = app.id, tag_count = tags.len(), "Extracted tags");
                                if let Err(error) = state.tags.update_tags(app.id, tags).await {
                                    error!(?error, app_id = app.id, "Failed to update tags");
                                }
                            }
                        }
                        Err(error) => {
                            error!(?error, app_id = app.id, "Failed to fetch workshop page");
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

fn extract_tags(app_id: u32, html: &str) -> Vec<Tag> {
    Html::parse_document(html)
        .select(&Selector::parse(".tag_label").unwrap())
        .filter_map(|node| {
            node.text()
                .collect::<String>()
                .split("\u{a0}")
                .next()
                .map(String::from)
        })
        .map(|text| Tag {
            app_id: app_id.into(),
            display_name: text.clone(),
            tag: text,
        })
        .collect::<Vec<_>>()
}
#[cfg(test)]
mod tests {
    use scraper::{Html, Selector};

    #[test]
    fn test_extract_tags() {
        let html = r#"
            <div class="panel">
                <div class="browseTitle">Browse By Tag</div>
                <div data-panel="{&quot;clickOnActivate&quot;:&quot;firstChild&quot;}" role="button" class="filterOption"><input type="checkbox" name="requiredtags[]" id="72498" value="Mod" class="inputTagsFilter" style="display: none"><label class="tag_label" for="72498">Mod&nbsp;<span class="tag_count">(31,497)</span></label></div>
                <div data-panel="{&quot;clickOnActivate&quot;:&quot;firstChild&quot;}" role="button" class="filterOption"><input type="checkbox" name="requiredtags[]" id="210323" value="Translation" class="inputTagsFilter" style="display: none"><label class="tag_label" for="210323">Translation&nbsp;<span class="tag_count">(11,250)</span></label></div>
                <div data-panel="{&quot;clickOnActivate&quot;:&quot;firstChild&quot;}" role="button" class="filterOption"><input type="checkbox" name="requiredtags[]" id="72499" value="Scenario" class="inputTagsFilter" style="display: none"><label class="tag_label" for="72499">Scenario&nbsp;<span class="tag_count">(7,504)</span></label></div>
                <div data-panel="{&quot;clickOnActivate&quot;:&quot;firstChild&quot;}" role="button" class="filterOption"><input type="checkbox" name="requiredtags[]" id="72890" value="0.14" class="inputTagsFilter" style="display: none"><label class="tag_label" for="72890">0.14&nbsp;<span class="tag_count">(915)</span></label></div>
                <div data-panel="{&quot;clickOnActivate&quot;:&quot;firstChild&quot;}" role="button" class="filterOption"><input type="checkbox" name="requiredtags[]" id="89609" value="0.15" class="inputTagsFilter" style="display: none"><label class="tag_label" for="89609">0.15&nbsp;<span class="tag_count">(560)</span></label></div>
                <div data-panel="{&quot;clickOnActivate&quot;:&quot;firstChild&quot;}" role="button" class="filterOption"><input type="checkbox" name="requiredtags[]" id="96812" value="0.16" class="inputTagsFilter" style="display: none"><label class="tag_label" for="96812">0.16&nbsp;<span class="tag_count">(915)</span></label></div>
                <div data-panel="{&quot;clickOnActivate&quot;:&quot;firstChild&quot;}" role="button" class="filterOption"><input type="checkbox" name="requiredtags[]" id="103979" value="0.17" class="inputTagsFilter" style="display: none"><label class="tag_label" for="103979">0.17&nbsp;<span class="tag_count">(1,095)</span></label></div>
                <div data-panel="{&quot;clickOnActivate&quot;:&quot;firstChild&quot;}" role="button" class="filterOption"><input type="checkbox" name="requiredtags[]" id="103980" value="0.18" class="inputTagsFilter" style="display: none"><label class="tag_label" for="103980">0.18&nbsp;<span class="tag_count">(1,572)</span></label></div>
                <div data-panel="{&quot;clickOnActivate&quot;:&quot;firstChild&quot;}" role="button" class="filterOption"><input type="checkbox" name="requiredtags[]" id="103981" value="0.19" class="inputTagsFilter" style="display: none"><label class="tag_label" for="103981">0.19&nbsp;<span class="tag_count">(544)</span></label></div>
                <div data-panel="{&quot;clickOnActivate&quot;:&quot;firstChild&quot;}" role="button" class="filterOption"><input type="checkbox" name="requiredtags[]" id="147418" value="1.0" class="inputTagsFilter" style="display: none"><label class="tag_label" for="147418">1.0&nbsp;<span class="tag_count">(7,809)</span></label></div>
                <div data-panel="{&quot;clickOnActivate&quot;:&quot;firstChild&quot;}" role="button" class="filterOption"><input type="checkbox" name="requiredtags[]" id="161748" value="1.1" class="inputTagsFilter" style="display: none"><label class="tag_label" for="161748">1.1&nbsp;<span class="tag_count">(8,728)</span></label></div>
                <div data-panel="{&quot;clickOnActivate&quot;:&quot;firstChild&quot;}" role="button" class="filterOption"><input type="checkbox" name="requiredtags[]" id="187443" value="1.2" class="inputTagsFilter" style="display: none"><label class="tag_label" for="187443">1.2&nbsp;<span class="tag_count">(11,219)</span></label></div>
                <div data-panel="{&quot;clickOnActivate&quot;:&quot;firstChild&quot;}" role="button" class="filterOption"><input type="checkbox" name="requiredtags[]" id="187571" value="1.3" class="inputTagsFilter" style="display: none"><label class="tag_label" for="187571">1.3&nbsp;<span class="tag_count">(16,059)</span></label></div>
                <div data-panel="{&quot;clickOnActivate&quot;:&quot;firstChild&quot;}" role="button" class="filterOption"><input type="checkbox" name="requiredtags[]" id="187570" value="1.4" class="inputTagsFilter" style="display: none"><label class="tag_label" for="187570">1.4&nbsp;<span class="tag_count">(20,679)</span></label></div>
                <div data-panel="{&quot;clickOnActivate&quot;:&quot;firstChild&quot;}" role="button" class="filterOption"><input type="checkbox" name="requiredtags[]" id="187569" value="1.5" class="inputTagsFilter" style="display: none"><label class="tag_label" for="187569">1.5&nbsp;<span class="tag_count">(22,632)</span></label></div>
                <div data-panel="{&quot;clickOnActivate&quot;:&quot;firstChild&quot;}" role="button" class="filterOption"><input type="checkbox" name="requiredtags[]" id="212893" value="1.6" class="inputTagsFilter" style="display: none"><label class="tag_label" for="212893">1.6&nbsp;<span class="tag_count">(20,979)</span></label></div>
            </div>
        "#;
        let document = Html::parse_document(html);
        let tags = document
            .select(&Selector::parse(".tag_label").unwrap())
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
