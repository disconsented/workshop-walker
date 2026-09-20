use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, OnceLock},
    time::Duration,
};

use ractor::{Actor, ActorProcessingErr, ActorRef, async_trait};
use reqwest::Client;
use snafu::{ResultExt, Whatever};
use surrealdb::{Surreal, engine::local::Db};
use tracing::{debug, error};

use crate::{
    application::tags_service::TagsService,
    db::{AppID, IAppID, ITagID, TagID, model::InternalTag, tags_repository::TagsSilo},
    steam::model::{GetTagCount, GetTagCountResponse, SteamRoot},
};

pub static TAGS_ACTOR: OnceLock<ActorRef<TagsMsg>> = OnceLock::new();

/// Actor responsible for handling workshop item tags operations
/// by delegating to the hexagonal `TagsService`.
///
/// The tag set this actor builds is the union of the tags on every item, so it
/// gets much larger than the tag list on the Steam workshop page. RimWorld
/// (294100) declares 16 tags, but its items also carry version tags that are
/// not declared (1.7, 1.8, 1.9) and free-form ones (Gameplay, Royalty, Utility,
/// Vanilla Expanded, Factions).
///
/// There is no API for the declared list. `IPublishedFileService` has only
/// `UpdateTags`, which is a publisher-only write, the possible read names give
/// 404, and appinfo has `store_tags` but no workshop tags. The only source is
/// the SSR JSON in the workshop browse page: GET
/// `https://steamcommunity.com/workshop/browse/?appid=<id>&l=english`, read
/// `window.SSR.renderContext`, JSON-decode it twice (`queryData` is a nested
/// JSON string), then find the query whose `queryKey[0]` starts with
/// `declared_tags_v`. Its `state.data` holds `readytouse_tags`, `mtx_tags`,
/// `collection_tags` and `merch_tags`, each a list of groups of
/// `{id, name, display_name, admin_only}`. This is a scrape. It breaks loudly
/// at the parse, so cache the last good list per app.
///
/// The option that needs no scrape: rank the tags by their true item count and
/// hide the tail in the UI. One call per tag gives the count:
/// `QueryFiles/v1/?appid=<id>&query_type=1&cursor=*&numperpage=1&totalonly=true
/// &requiredtags[0]=<tag>`. Use `requiredtags[0]`, not `requiredtags`: the bare
/// name is ignored and the response holds the unfiltered total. For 294100 a
/// cutoff near 100 items keeps all 16 declared tags and drops the junk
/// (Gameplay 14, Utility 5, Royalty 3, Vanilla Expanded 2, Factions 1).
pub struct TagsActor;

/// Actor initialization arguments.
pub struct TagsArgs {
    pub database: Surreal<Db>,
    pub client: Client,
    pub steam_token: Arc<String>,
}

/// Internal state for the actor. Holds the service instance.
pub struct TagsState {
    service: TagsService<TagsSilo>,
    tags_cache: HashMap<IAppID, HashSet<InternalTag>>,
    client: Client,
    steam_token: Arc<String>,
}

/// Messages handled by `TagsActor`.
pub enum TagsMsg {
    AddTagToApp(IAppID, Vec<InternalTag>),
    Clear,
    UpdateCount(IAppID, ITagID),
}

/// TagsActor keeps an internal cache of tags, updating the database when it
/// thinks there are any new ones. This is intentionally not perfect, for the
/// sake of performance, I'm making a deliberate trade-off to sometimes upsert
/// redundantly instead of adding an extra query for _every_ item.
///
/// https://api.steampowered.com/IPublishedFileService/QueryFiles/v1/?key=YOUR_KEY&query_type=1&cursor=*&numperpage=1&appid=294100&requiredtags[0]=Translation&totalonly=true
#[async_trait]
impl Actor for TagsActor {
    type Arguments = TagsArgs;
    type Msg = TagsMsg;
    type State = TagsState;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        TAGS_ACTOR.get_or_init(|| myself.clone());
        // Clear the cache daily
        myself.send_interval(Duration::from_hours(24), || TagsMsg::Clear);
        Ok(TagsState {
            service: TagsService::new(TagsSilo::new(args.database)),
            tags_cache: HashMap::new(),
            client: args.client,
            steam_token: args.steam_token,
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            TagsMsg::AddTagToApp(appid, tags) => {
                let entry = state.tags_cache.entry(appid.clone()).or_default();
                let new_tags = tags.into_iter().fold(vec![], |mut acc, tag| {
                    if entry.insert(tag.clone()) {
                        acc.push(tag.clone());
                        let _ = myself.send_message(TagsMsg::UpdateCount(appid.clone(), tag.id));
                    }
                    acc
                });
                if !new_tags.is_empty()
                    && let Err(error) = state.service.update_tags(appid.clone(), new_tags).await
                {
                    error!(?error, ?appid, "Failed to update tags");
                }
            }
            TagsMsg::Clear => state.tags_cache.clear(),
            TagsMsg::UpdateCount(appid, tag) => {
                if let Err(error) = Self::run_get_count(state, appid, tag).await {
                    error!(?error, "Failed to update known members count");
                }
            }
        }
        Ok(())
    }
}

impl TagsActor {
    // N.B. there's a possible bug here, because steam does weird things with
    // totals when you _dont_ specify an app, we may get incorrect tag counts
    // for an app
    async fn run_get_count(
        state: &mut TagsState,
        app: IAppID,
        tag: ITagID,
    ) -> Result<(), Whatever> {
        let tag = TagID::try_from(tag).whatever_context("TagID from ITagID")?;
        let app = AppID::try_from(app).whatever_context("AppID from IAppID")?;
        let request = GetTagCount {
            tag_id: tag.clone().into(),
            app_id: i64::from(app) as u32,
        }
        .into_request(&state.client, &state.steam_token)
        .whatever_context("into_request")?;
        let response = state
            .client
            .execute(request)
            .await
            .whatever_context("Sending get page request")?;

        let json = response
            .json::<SteamRoot<GetTagCountResponse>>()
            .await
            .whatever_context("request body")?;

        if let Err(error) = state
            .service
            .set_tag_known_members(tag.clone().into(), json.response.total)
            .await
        {
            error!(?error, ?tag, "Failed to update tag's known member count");
        }
        debug!(
            ?tag,
            known_members = json.response.total,
            "updated tag known member count"
        );
        Ok(())
    }
}
