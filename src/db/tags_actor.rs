use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

use ractor::{Actor, ActorProcessingErr, ActorRef, async_trait};
use surrealdb::{Surreal, engine::local::Db};
use tracing::error;

use crate::{
    application::tags_service::TagsService,
    db::{IAppID, model::InternalTag, tags_repository::TagsSilo},
};

pub static TAGS_ACTOR: OnceLock<ActorRef<TagsMsg>> = OnceLock::new();

/// Actor responsible for handling workshop item tags operations
/// by delegating to the hexagonal `TagsService`.
pub struct TagsActor;

/// Actor initialization arguments.
pub struct TagsArgs {
    pub database: Surreal<Db>,
}

/// Internal state for the actor. Holds the service instance.
pub struct TagsState {
    service: TagsService<TagsSilo>,
    tags_cache: HashMap<IAppID, HashSet<InternalTag>>,
}

/// Messages handled by `TagsActor`.
pub enum TagsMsg {
    AddTagToApp(IAppID, Vec<InternalTag>),
}

/// TagsActor keeps an internal cache of tags, updating the database when it
/// thinks there are any new ones. This is intentionally not perfect, for the
/// sake of performance, I'm making a deliberate trade-off to sometimes upsert
/// redundantly instead of adding an extra query for _every_ item.
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
        TAGS_ACTOR.get_or_init(|| myself);
        Ok(TagsState {
            service: TagsService::new(TagsSilo::new(args.database)),
            tags_cache: HashMap::new(),
        })
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            TagsMsg::AddTagToApp(appid, tags) => {
                let entry = state.tags_cache.entry(appid.clone()).or_default();
                let new_tags = tags.into_iter().fold(vec![], |mut acc, tag| {
                    if entry.insert(tag.clone()) {
                        acc.push(tag);
                    }
                    acc
                });
                if let Err(error) = state.service.update_tags(appid.clone(), new_tags).await {
                    error!(?error, ?appid, "Failed to update tags");
                }
            }
        }
        Ok(())
    }
}
