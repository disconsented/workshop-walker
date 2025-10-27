use ractor::{Actor, ActorProcessingErr, ActorRef, async_trait};
use snafu::{ResultExt, Whatever};
use surrealdb::{
    RecordId, Surreal,
    engine::local::Db,
    sql::{Data, Value, statements::InsertStatement, to_value},
};
use tracing::error;

use crate::{
    db::model::{Dependencies, WorkshopItem},
    processing::{
        bb_actor::BBMsg,
        join_process_actor::{JoinProcessActor, JoinProcessArgs, JoinProcessMsg},
        language_actor::LanguageMsg,
    },
    steam::model::{Child, IPublishedResponse, IPublishedStruct, SteamRoot},
};

pub struct ItemUpdateActor {}

pub struct ItemUpdateArgs {
    pub language_actor: ActorRef<LanguageMsg>,
    pub bb_actor: ActorRef<BBMsg>,
    pub database: Surreal<Db>,
}
pub struct ItemUpdateState {
    language_actor: ActorRef<LanguageMsg>,
    bb_actor: ActorRef<BBMsg>,
    database: Surreal<Db>,
}

pub enum ItemUpdateMsg {
    RawProcess(SteamRoot<IPublishedResponse>),
    Process(IPublishedStruct),
    Upsert((WorkshopItem<RecordId>, Vec<Child>)),
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
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            ItemUpdateMsg::Process(data) => {
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
                if let Err(e) = insert_data(&state.database, item, children).await {
                    error!("{e:?}");
                }
            }
            ItemUpdateMsg::RawProcess(steam_root) => {
                for file in steam_root.response.publishedfiledetails {
                    match serde_json::from_value(file) {
                        Ok(file) => {
                            myself.send_message(ItemUpdateMsg::Process(file))?;
                        }
                        Err(error) => {
                            error!(?error, "deserializing raw files");
                        }
                    }
                }
            }
        }

        Ok(())
    }
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
        error!("{sql}");
        error!("{errors:#?}");
        panic!("Errors: ")
    }

    Ok(())
}
