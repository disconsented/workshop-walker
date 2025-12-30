use std::mem::take;

use ractor::{Actor, ActorProcessingErr, ActorRef, async_trait, call};
use snafu::{OptionExt, Whatever};
use surrealdb::RecordId;
use tracing::error;

use crate::{
    db::{item_update_actor::ItemUpdateMsg, model, model::WorkshopItem},
    processing::{
        bb_actor::BBMsg,
        language_actor::{DetectedLanguage, LanguageMsg},
    },
    steam::model::IPublishedStruct,
};

/// Ephemeral actor, only used to coordinate tasks without tying up the greater
/// `ItemUpdateActor`
pub struct JoinProcessActor {}

pub struct JoinProcessArgs {
    pub item_update: ActorRef<ItemUpdateMsg>,
    pub language: ActorRef<LanguageMsg>,
    pub bb: ActorRef<BBMsg>,
}
pub struct JoinProcessState {
    item_update: ActorRef<ItemUpdateMsg>,
    language: ActorRef<LanguageMsg>,
    bb: ActorRef<BBMsg>,
}

pub enum JoinProcessMsg {
    Process(IPublishedStruct),
}
#[async_trait]
impl Actor for JoinProcessActor {
    type Arguments = JoinProcessArgs;
    type Msg = JoinProcessMsg;
    type State = JoinProcessState;

    async fn pre_start(
        &self,
        _: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(Self::State {
            item_update: args.item_update,
            language: args.language,
            bb: args.bb,
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            JoinProcessMsg::Process(mut data) => {
                let description = take(&mut data.file_description).unwrap_or_default();
                let languages = call!(state.language, LanguageMsg::Detect, description.clone())?;
                let description = call!(state.bb, BBMsg::Process, description)?;
                let children = take(&mut data.children);
                match WorkshopItem::try_new(data, languages, description) {
                    Ok(item) => {
                        state
                            .item_update
                            .send_message(ItemUpdateMsg::MaybeQueueMl((item, children)))?;
                    }
                    Err(error) => {
                        error!(%error, "Creating new item");
                    }
                }
            }
        }
        myself.stop(None);
        Ok(())
    }
}

impl WorkshopItem<RecordId> {
    fn try_new(
        data: IPublishedStruct,
        languages: Vec<DetectedLanguage>,
        description: String,
    ) -> Result<Self, Whatever> {
        let app_id = data
            .consumer_appid
            .whatever_context("Missing app id")
            .inspect_err(|_| error!(?data, "creating new item"))? as u64;
        Ok(Self {
            appid: app_id,
            author: data.creator.whatever_context("Missing author")?,
            languages,
            description,
            id: RecordId::from_table_key("workshop_items", data.publishedfileid),
            title: data.title.whatever_context("Missing title")?,
            preview_url: data.preview_url,
            last_updated: data.time_updated.unwrap_or_default() as _,
            tags: data
                .tags
                .iter()
                .cloned()
                .map(|tag| model::Tag {
                    app_id,
                    display_name: tag.display_name,
                    tag: tag.tag,
                })
                .collect::<Vec<_>>(),
            score: data.vote_data.map(|votes| votes.score).unwrap_or_default(),
            properties: vec![],
        })
    }
}
