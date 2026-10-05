use std::{collections::HashMap, sync::Arc};

use ractor::{Actor, ActorProcessingErr, ActorRef, async_trait, call};
use snafu::Whatever;
use surrealdb::{Surreal, engine::local::Db};
use tokio::{task, task::JoinHandle};
use tracing::{debug, error, trace};

use crate::{
    application::items_service::ItemsService,
    db::{
        IItemID,
        items_repository::ItemsSilo,
        model::{Class, InternalSource, Status},
        properties_actor::PropertiesMsg,
    },
    domain::properties::{InternalNewProperty, PropertiesError},
    processing::llama_actor::LlamaMsg,
};

pub struct MLQueueActor;

pub struct MLQueueArgs {
    pub extractor: ActorRef<LlamaMsg>,
    pub property_actor: ActorRef<PropertiesMsg>,
    pub database: Surreal<Db>,
}

pub struct MLQueueState {
    extractor: ActorRef<LlamaMsg>,
    property_actor: ActorRef<PropertiesMsg>,
    task: Option<JoinHandle<()>>,
    queue: HashMap<IItemID, (String, String)>,
    items_service: Arc<ItemsService<ItemsSilo>>,
}

pub enum MLQueueMsg {
    /// Enqueue a workshop item id (record id) to be sent to the ML extractor
    Queue(IItemID, String, String),
    Finished(Result<(), Whatever>),
}

#[async_trait]
impl Actor for MLQueueActor {
    type Arguments = MLQueueArgs;
    type Msg = MLQueueMsg;
    type State = MLQueueState;

    async fn pre_start(
        &self,
        _: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(MLQueueState {
            extractor: args.extractor,
            property_actor: args.property_actor,
            task: None,
            queue: HashMap::default(),
            items_service: Arc::new(ItemsService::new(ItemsSilo::new(args.database))),
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            MLQueueMsg::Queue(id, title, description) => {
                trace!(
                    running_task = state.task.is_some(),
                    "Queue message received"
                );
                if state.task.is_some() {
                    state.queue.insert(id, (title, description));
                } else {
                    queue(myself, state, id, title, description);
                }
            }
            MLQueueMsg::Finished(result) => {
                if let Err(error) = result {
                    error!(?error, "Error occurred processing ML");
                }
                state.task = None;
                queue_next(myself, state);
                debug!(items = state.queue.len(), "Items queued for ML");
            }
        }
        Ok(())
    }
}

fn queue(
    myself: ActorRef<MLQueueMsg>,
    state: &mut MLQueueState,
    id: IItemID,
    title: String,
    description: String,
) {
    let extractor = state.extractor.clone();
    let property_actor = state.property_actor.clone();
    let items_service = state.items_service.clone();
    let task = task::spawn(async move {
        debug!(?id, "starting ML task");
        let result = process_one(
            extractor,
            property_actor,
            items_service,
            id,
            title,
            description,
        )
        .await;
        let _ = myself.send_message(MLQueueMsg::Finished(result));
    });

    state.task = Some(task);
}
fn queue_next(myself: ActorRef<MLQueueMsg>, state: &mut MLQueueState) {
    if let Some(id) = state.queue.iter().next().map(|(id, _)| id.clone())
        && let Some((title, description)) = state.queue.remove(&id)
    {
        queue(myself, state, id, title, description);
    }
}

async fn process_one(
    extractor: ActorRef<LlamaMsg>,
    property_actor: ActorRef<PropertiesMsg>,
    items_service: Arc<ItemsService<ItemsSilo>>,
    workshop_item_id: IItemID,
    title: String,
    description: String,
) -> Result<(), Whatever> {
    // Call the extractor via RPC using ractor::call! macro
    match call!(extractor, |reply| LlamaMsg::Process {
        title,
        description,
        rpc_reply_port: reply
    }) {
        Ok(Ok(props)) => {
            debug!(?workshop_item_id, ?props, "ML extraction completed");

            if let Err(error) = items_service
                .update_ml_last_run(workshop_item_id.clone())
                .await
            {
                error!(?error, "Error occurred updating timestamp for ml last run");
            }
            for (class, value) in props
                .genres
                .into_iter()
                .map(|v| (Class::Genre, v))
                .chain(props.themes.into_iter().map(|v| (Class::Theme, v)))
                .chain(props.types.into_iter().map(|v| (Class::Type, v)))
                .chain(props.features.into_iter().map(|v| (Class::Feature, v)))
            {
                match call!(property_actor, |reply| PropertiesMsg::NewProperty(
                    InternalNewProperty {
                        workshop_item: workshop_item_id.clone(),
                        class: class.clone(),
                        value: value.clone(),
                        note: None,
                    },
                    InternalSource::System,
                    Status::Accepted,
                    reply
                )) {
                    Ok(Ok(..)) => {
                        debug!(?workshop_item_id, %class, %value, "Inserted new property");
                    }
                    Ok(Err(PropertiesError::Conflict)) => {
                        debug!(?workshop_item_id, %class, %value, "Property conflict");
                    }
                    Ok(Err(error)) => {
                        error!(?error, ?workshop_item_id,  %class, %value, "Inserting new property");
                    }
                    Err(_) => (),
                }
            }
        }
        Ok(Err(error)) => {
            error!(?error, ?workshop_item_id, "ML extraction failed");
        }
        Err(error) => {
            error!(?error, ?workshop_item_id, "ML extractor RPC failed");
        }
    }
    Ok(())
}
