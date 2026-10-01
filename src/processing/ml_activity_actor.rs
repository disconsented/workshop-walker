use std::{num::NonZeroUsize, sync::OnceLock};

use lru::LruCache;
use ractor::{Actor, ActorProcessingErr, ActorRef, async_trait};
use tracing::debug;

use crate::db::{IItemID, item_update_actor::ItemUpdateMsg};

pub static ML_ACTIVITY_ACTOR: OnceLock<ActorRef<MLActivityMsg>> = OnceLock::new();
// Pulled out of thin air
const CACHE_SIZE: NonZeroUsize = const { NonZeroUsize::new(1024).expect("Invalid cache size") };

/// Use activity signals from search/get item to figure out if an item should be
/// considered for running ML on it. We're assuming that most of the time this
/// would be redundant, given expectations for users to look at the same items
/// or the same search results.
pub struct MLActivityActor;

pub struct MLActivityArgs {
    pub item_update_actor: ActorRef<ItemUpdateMsg>,
}

pub struct MLActivityState {
    cache: LruCache<IItemID, ()>,
    item_update_actor: ActorRef<ItemUpdateMsg>,
}

pub enum MLActivityMsg {
    Signal(IItemID),
    SignalBatch(Vec<IItemID>),
}

#[async_trait]
impl Actor for MLActivityActor {
    type Arguments = MLActivityArgs;
    type Msg = MLActivityMsg;
    type State = MLActivityState;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        ML_ACTIVITY_ACTOR.get_or_init(|| myself);
        Ok(MLActivityState {
            cache: LruCache::new(CACHE_SIZE),
            item_update_actor: args.item_update_actor,
        })
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            Self::Msg::Signal(id) => {
                debug!(?id, "got hint");
                MLActivityActor::check_or_push(state, id);
            }
            Self::Msg::SignalBatch(ids) => {
                debug!("got bulk hint");
                for id in ids {
                    MLActivityActor::check_or_push(state, id);
                }
            }
        }
        Ok(())
    }
}

impl MLActivityActor {
    fn check_or_push(state: &mut MLActivityState, id: IItemID) {
        if state.cache.put(id.clone(), ()).is_none() {
            let _ = state
                .item_update_actor
                .send_message(ItemUpdateMsg::MaybeQueue(id));
        }
    }
}
