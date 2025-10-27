use ractor::{Actor, ActorProcessingErr, ActorRef, async_trait};
use snafu::{ResultExt, Whatever};
use surrealdb::{
    RecordId, Surreal,
    engine::local::Db,
    sql::{Data, Value, statements::InsertStatement, to_value},
};
use tracing::{debug, error};

use crate::{
    db::model::{Dependencies, WorkshopItem},
    processing::{
        bb_actor::BBMsg,
        join_process_actor::{JoinProcessActor, JoinProcessArgs, JoinProcessMsg},
        language_actor::{DetectedLanguage, LanguageMsg},
        ml_queue_actor::MLQueueMsg,
    },
    steam::model::{Child, IPublishedResponse, IPublishedStruct, SteamRoot},
};

pub struct ItemUpdateActor {}

pub struct ItemUpdateArgs {
    pub language_actor: ActorRef<LanguageMsg>,
    pub bb_actor: ActorRef<BBMsg>,
    pub database: Surreal<Db>,
    pub ml_queue: Option<ActorRef<MLQueueMsg>>, // optional ML queue actor
}
pub struct ItemUpdateState {
    language_actor: ActorRef<LanguageMsg>,
    bb_actor: ActorRef<BBMsg>,
    database: Surreal<Db>,
    ml_queue: Option<ActorRef<MLQueueMsg>>,
}

pub enum ItemUpdateMsg {
    DeserializeRawFiles(SteamRoot<IPublishedResponse>),
    MainlineProcessing(IPublishedStruct),
    Upsert((WorkshopItem<RecordId>, Vec<Child>)),
    MaybeQueueMl((WorkshopItem<RecordId>, Vec<Child>)),
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
            database: args.database,
            language_actor: args.language_actor,
            bb_actor: args.bb_actor,
            ml_queue: args.ml_queue,
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
                    match serde_json::from_value(file) {
                        Ok(file) => {
                            myself.send_message(ItemUpdateMsg::MainlineProcessing(file))?;
                        }
                        Err(error) => {
                            error!(?error, "deserializing raw files");
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
            ItemUpdateMsg::MaybeQueueMl((item, children)) => {
                if let Err(error) = maybe_queue_ml(&state.database, &state.ml_queue, &item).await {
                    error!(?error, id = %item.id, "queuing ML work (message)");
                }
                if myself
                    .send_message(ItemUpdateMsg::Upsert((item, children)))
                    .is_err()
                {
                    error!("forwarding work to upsert");
                }
            }
            ItemUpdateMsg::Upsert((item, children)) => {
                let title = item.title.clone();
                let item_id = item.id.clone();
                if let Err(error) = insert_data(&state.database, item, children).await {
                    error!(?error, title, %item_id, "upserting item");
                }
            }
        }

        Ok(())
    }
}
/// Attempt to extract data from posts text using an LLM under the following
/// conditions:
///
/// 1. We've enabled the functionality
/// 2. The detected languages include english, as the model doesn't work well
///    otherwise
/// 3. The item's `last_updated` has changed, using this as a cheap proxy for
///    detecting changes
/// 4. Finally, the description has changed, we'll likely get the same result
///    for the same input
async fn maybe_queue_ml(
    db: &Surreal<Db>,
    ml_queue: &Option<ActorRef<MLQueueMsg>>,
    item: &WorkshopItem<RecordId>,
) -> crate::Result<(), Whatever> {
    if let Some(queue) = ml_queue {
        let mut resp = db
            .query("SELECT last_updated, description FROM $id")
            .bind(("id", item.id.clone()))
            .await
            .whatever_context("querying last_updated for ML queue check")?;
        let old_last_updated: Option<u64> = resp
            .take((0, "last_updated"))
            .whatever_context("taking last_updated for ML queue check")?;
        let old_description: Option<String> = resp
            .take((0, "description"))
            .whatever_context("taking description for ML queue check")?;
        let old_description = old_description.unwrap_or_default();
        let outdated = old_last_updated != Some(item.last_updated);
        let description_changed = &old_description != &item.description;
        let viable_language = item.languages.contains(&DetectedLanguage::English);
        // We don't want to waste our resources on extracting
        if viable_language && outdated && description_changed {
            debug!(
                name = item.title,
                outdated, description_changed, "Item is being processed for extraction"
            );
            let _ = queue.send_message(MLQueueMsg::Process(item.id.clone()));
        }
    }
    Ok(())
}

async fn insert_data(
    db: &Surreal<Db>,
    mut item: WorkshopItem<RecordId>,
    children: Vec<Child>,
) -> crate::Result<(), Whatever> {
    let tags = std::mem::take(&mut item.tags);
    let id = item.id.clone();
    let insert_tags = {
        let mut stmt = InsertStatement::default();
        stmt.into = Some(Value::Table("tags".into()));
        stmt.data = Data::SingleExpression(to_value(tags.clone()).unwrap());
        stmt.ignore = true;
        stmt
    };

    let insert_item_deps = {
        let mut stmt = InsertStatement::default();
        stmt.relation = true;

        stmt.into = Some(Value::Table("item_dependencies".into()));
        let data = children
            .into_iter()
            .map(|child| {
                let dep_id = RecordId::from_table_key("workshop_items", child.publishedfileid);
                to_value(Dependencies {
                    id: RecordId::from_table_key(
                        "item_dependencies",
                        vec![item.id.clone().into(), dep_id.clone().into()],
                    ),
                    this: item.id.clone(),
                    dependency: dep_id,
                })
                .unwrap()
            })
            .collect::<Vec<_>>();
        stmt.data = Data::SingleExpression(Value::Array(data.into()));
        stmt.ignore = true;
        stmt
    };

    let upsert_item = {
        let mut stmt = InsertStatement::default();
        stmt.data = Data::SingleExpression(to_value(item).unwrap());
        stmt.into = Some(Value::Table("workshop_items".into()));
        stmt.ignore = true;
        stmt
    };

    let query = db
        .query("BEGIN TRANSACTION")
        .query(insert_tags)
        .query(upsert_item)
        .query(insert_item_deps)
        .query("UPDATE $id SET tags=$tags")
        .bind(("id", id))
        .bind((
            "tags",
            tags.iter()
                .map(|tag| RecordId::from_table_key("tags", tag.tag.clone()))
                .collect::<Vec<_>>(),
        ))
        .query("COMMIT");
    let sql = format!("{query:#?}");
    let mut response = query.await.whatever_context("big insert query")?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        error!(query = sql, ?errors, "inserting data");
    }

    Ok(())
}
