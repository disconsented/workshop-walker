use ractor::{Actor, ActorProcessingErr, ActorRef, async_trait};
use surrealdb::{Surreal, engine::local::Db};
use tracing::error;

use crate::{
    application::items_service::ItemsService,
    db::{IItemID, items_repository::ItemsSilo, model::InternalWorkshopItem, tags_actor::TagsMsg},
    processing::{
        bb_actor::BBMsg,
        join_process_actor::{JoinProcessActor, JoinProcessArgs, JoinProcessMsg},
        language_actor::LanguageMsg,
        ml_queue_actor::MLQueueMsg,
    },
    steam::{
        model::{Child, EResult, IPublishedResponse, IPublishedStruct, SteamRoot},
        steam_user_actor::SteamUserMsg,
    },
};

pub struct ItemUpdateActor {}

pub struct ItemUpdateArgs {
    pub language_actor: ActorRef<LanguageMsg>,
    pub bb_actor: ActorRef<BBMsg>,
    pub steam_user_actor: ActorRef<SteamUserMsg>,
    pub database: Surreal<Db>,
    pub ml_queue: Option<ActorRef<MLQueueMsg>>, // optional ML queue actor
    pub tags_actor: ActorRef<TagsMsg>,
}
pub struct ItemUpdateState {
    language_actor: ActorRef<LanguageMsg>,
    bb_actor: ActorRef<BBMsg>,
    steam_user_actor: ActorRef<SteamUserMsg>,
    items_service: ItemsService<ItemsSilo>,
    ml_queue: Option<ActorRef<MLQueueMsg>>,
    tags_actor: ActorRef<TagsMsg>,
}

pub enum ItemUpdateMsg {
    DeserializeRawFiles(SteamRoot<IPublishedResponse>),
    MainlineProcessing(IPublishedStruct),
    Upsert((InternalWorkshopItem, Vec<Child>)),
    /// Intended to be invoked from user intent signals
    MaybeQueue(IItemID),
}
#[async_trait]
impl Actor for ItemUpdateActor {
    type Arguments = ItemUpdateArgs;
    type Msg = ItemUpdateMsg;
    type State = ItemUpdateState;

    async fn pre_start(
        &self,
        _: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(Self::State {
            items_service: ItemsService::new(ItemsSilo::new(args.database)),
            language_actor: args.language_actor,
            bb_actor: args.bb_actor,
            steam_user_actor: args.steam_user_actor,
            ml_queue: args.ml_queue,
            tags_actor: args.tags_actor,
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            ItemUpdateMsg::DeserializeRawFiles(steam_root) => {
                for file in steam_root.response.publishedfiledetails {
                    match serde_json::from_value::<IPublishedStruct>(file) {
                        Ok(file) => {
                            if file.result == EResult::OK as i32 {
                                myself.send_message(ItemUpdateMsg::MainlineProcessing(file))?;
                            }
                        }
                        Err(error) => {
                            error!(?error, "deserializing raw file");
                        }
                    }
                }
            }
            ItemUpdateMsg::MainlineProcessing(data) => {
                let (join_process_actor, _) = Actor::spawn(
                    None,
                    JoinProcessActor {},
                    JoinProcessArgs {
                        item_update: myself.clone(),
                        language: state.language_actor.clone(),
                        bb: state.bb_actor.clone(),
                    },
                )
                .await?;

                join_process_actor.send_message(JoinProcessMsg::Process(data))?;
            }
            ItemUpdateMsg::Upsert((item, children)) => {
                let title = item.title.clone();
                let item_id = item.id.clone();

                let _ = state
                    .tags_actor
                    .send_message(TagsMsg::AddTagToApp(item.app.clone(), item.tags.clone()));

                let _ = state
                    .steam_user_actor
                    .send_message(SteamUserMsg::Fetch(item.author.id.clone()));

                if let Some(ml_queue) = &state.ml_queue
                    && let Ok(true) = state.items_service.should_queue_ml(&item).await
                {
                    let _ = ml_queue.send_message(MLQueueMsg::Queue(
                        item.id.clone(),
                        item.title.clone(),
                        item.description.clone(),
                    ));
                }

                if let Err(error) = state.items_service.insert_data(item, children).await {
                    error!(?error, title, ?item_id, "upserting item");
                }
            }
            ItemUpdateMsg::MaybeQueue(id) => {
                if let Some(ml_queue) = &state.ml_queue {
                    match state.items_service.get_item(id.clone()).await {
                        Ok(item) => {
                            if let Ok(true) = state
                                .items_service
                                .should_queue_ml(&item)
                                .await
                                .inspect_err(|error| {
                                    error!(?error, "checking if should queue ML for hint");
                                })
                            {
                                let _ = ml_queue.send_message(MLQueueMsg::Queue(
                                    item.id.clone(),
                                    item.title.clone(),
                                    item.description.clone(),
                                ));
                            }
                        }
                        Err(error) => {
                            error!(?error, "getting item");
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
