use std::num::ParseIntError;

use ractor::{Actor, ActorProcessingErr, ActorRef, async_trait};
use snafu::{ResultExt, Whatever};
use surrealdb::{Surreal, engine::local::Db};
use surrealdb_core::sql::{
    Expr,
    data::Data,
    statements::{InsertStatement, UpsertStatement},
};
use surrealdb_types::{SurrealValue, Value};
use tracing::{Instrument, debug, debug_span, error, info_span};

use crate::{
    db::{
        IItemID,
        model::{InsertableWorkshopItem, InternalWorkshopItem},
        tags_actor::TagsMsg,
    },
    processing::{
        bb_actor::BBMsg,
        join_process_actor::{JoinProcessActor, JoinProcessArgs, JoinProcessMsg},
        language_actor::{DetectedLanguage, LanguageMsg},
        ml_queue_actor::MLQueueMsg,
    },
    steam::{
        model::{Child, IPublishedResponse, IPublishedStruct, SteamRoot},
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
    database: Surreal<Db>,
    ml_queue: Option<ActorRef<MLQueueMsg>>,
    tags_actor: ActorRef<TagsMsg>,
}

pub enum ItemUpdateMsg {
    DeserializeRawFiles(SteamRoot<IPublishedResponse>),
    MainlineProcessing(IPublishedStruct),
    Upsert((InternalWorkshopItem, Vec<Child>)),
    MaybeQueueMl((InternalWorkshopItem, Vec<Child>)),
}
#[async_trait]
impl Actor for ItemUpdateActor {
    type Arguments = ItemUpdateArgs;
    type Msg = ItemUpdateMsg;
    type State = ItemUpdateState;

    #[tracing::instrument(level = "debug", skip_all)]
    async fn pre_start(
        &self,
        _: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(Self::State {
            database: args.database,
            language_actor: args.language_actor,
            bb_actor: args.bb_actor,
            steam_user_actor: args.steam_user_actor,
            ml_queue: args.ml_queue,
            tags_actor: args.tags_actor,
        })
    }

    /// Each arm makes its own span, and each one is a root. A message arrives
    /// on its own task, so there is no caller span to hang it from. `item.id`
    /// is the field to search on to follow one item across the stages.
    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            ItemUpdateMsg::DeserializeRawFiles(steam_root) => {
                let files = steam_root.response.publishedfiledetails;
                let batch = info_span!(
                    parent: None,
                    "deserialize raw files",
                    batch.size = files.len(),
                    batch.failed = tracing::field::Empty,
                );
                let _entered = batch.enter();
                let mut failed = 0_usize;
                for file in files {
                    match serde_json::from_value::<IPublishedStruct>(file) {
                        Ok(file) => {
                            myself.send_message(ItemUpdateMsg::MainlineProcessing(file))?;
                        }
                        Err(error) => {
                            failed += 1;
                            error!(?error, "deserializing raw file");
                        }
                    }
                }
                batch.record("batch.failed", failed);
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
                .instrument(debug_span!("spawn join process", item.id = %data.publishedfileid))
                .await?;

                join_process_actor.send_message(JoinProcessMsg::Process(data))?;
            }
            ItemUpdateMsg::MaybeQueueMl((workshop_item, children)) => {
                if let Err(error) =
                    maybe_queue_ml(&state.database, state.ml_queue.as_ref(), &workshop_item)
                        .instrument(debug_span!("maybe queue ml", item.id = ?workshop_item.id.key))
                        .await
                {
                    error!(?error, id = ?workshop_item.id, "queuing ML work (message)");
                }
                if myself
                    .send_message(ItemUpdateMsg::Upsert((workshop_item, children)))
                    .is_err()
                {
                    error!("forwarding work to upsert");
                }
            }
            ItemUpdateMsg::Upsert((workshop_item, children)) => {
                let span = debug_span!("insert item", item.id = ?workshop_item.id.key);
                let title = workshop_item.title.clone();
                let item_id = workshop_item.id.clone();

                let _ = state.tags_actor.send_message(TagsMsg::AddTagToApp(
                    workshop_item.app.clone(),
                    workshop_item.tags.clone(),
                ));

                let _ = state
                    .steam_user_actor
                    .send_message(SteamUserMsg::Fetch(workshop_item.author.id.clone()));

                if let Err(error) = insert_data(&state.database, workshop_item, children)
                    .instrument(span)
                    .await
                {
                    error!(?error, title, ?item_id, "upserting item");
                }
            }
        }

        Ok(())
    }
}
#[tracing::instrument(level = "debug", skip(db, ml_queue, item))]
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
    ml_queue: Option<&ActorRef<MLQueueMsg>>,
    item: &InternalWorkshopItem,
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
        let description_changed = old_description != item.description;
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

#[tracing::instrument(level = "debug", skip(db, item, children))]
async fn insert_data(
    db: &Surreal<Db>,
    mut item: InternalWorkshopItem,
    children: Vec<Child>,
) -> crate::Result<(), Whatever> {
    let tags = std::mem::take(&mut item.tags);
    let id = item.id.clone();

    let insert_item_deps = {
        children
            .into_iter()
            .map(|child| {
                let dep_id = IItemID::from(child.publishedfileid.parse::<i64>()?);
                Ok(InsertStatement {
                    into: Some(Expr::Table("item_dependencies".into())),
                    data: Data::SingleExpression(Expr::from_public_value(Value::Object(
                        vec![
                            // Another "fun" surreal detail, insert does conflict on the ID... not
                            // the actual relation despite their being a unique index
                            (
                                "id".into(),
                                [id.clone().into_value(), dep_id.clone().into_value()].into_value(),
                            ),
                            ("in".into(), item.id.clone().into_value()),
                            ("out".into(), dep_id.into_value()),
                        ]
                        .into_iter()
                        .collect(),
                    ))),
                    ignore: true,
                    relation: true,
                    ..Default::default()
                })
            })
            .collect::<Result<Vec<_>, ParseIntError>>()
            .whatever_context("parsing publishedfileids")?
    };

    let upsert_item = UpsertStatement {
        data: Some(Data::ReplaceExpression(Expr::from_public_value(
            InsertableWorkshopItem {
                app: item.app,
                author: item.author.id,
                description: item.description,
                id: item.id,
                languages: item.languages,
                last_updated: item.last_updated,
                preview_url: item.preview_url,
                title: item.title,
                score: item.score,
                tags: tags.into_iter().map(|tag| tag.id).collect::<Vec<_>>(),
            }
            .into_value(),
        ))),
        what: vec![Expr::Table("workshop_items".into())],
        ..Default::default()
    };

    let mut query = db.query("BEGIN").query(upsert_item);
    for insert_dep in insert_item_deps {
        query = query.query(insert_dep);
    }
    let query = query.bind(("id", id)).query("COMMIT");
    let sql = format!("{query:?}");
    let mut response = query.await.whatever_context("big insert query")?;

    let errors = response.take_errors();
    if !errors.is_empty() {
        error!(?errors, sql, "inserting data");
    }

    Ok(())
}
