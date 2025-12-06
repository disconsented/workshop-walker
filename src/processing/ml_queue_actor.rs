use classification::actor::ExtractionMsg;
use ractor::{Actor, ActorProcessingErr, ActorRef, async_trait, call};
use snafu::{ResultExt, Whatever};
use surrealdb::{RecordId, Surreal, engine::local::Db};
use tracing::{debug, error, info};

use crate::{
    db::{
        model::{Class, Source},
        properties_actor::PropertiesMsg,
    },
    domain::properties::NewProperty,
};

pub struct MLQueueActor;

pub struct MLQueueArgs {
    pub database: Surreal<Db>,
    pub extractor: ActorRef<ExtractionMsg>,
    pub property_actor: ActorRef<PropertiesMsg>,
}

pub struct MLQueueState {
    database: Surreal<Db>,
    extractor: ActorRef<ExtractionMsg>,
    property_actor: ActorRef<PropertiesMsg>,
}

pub enum MLQueueMsg {
    /// Enqueue a workshop item id (record id) to be sent to the ML extractor
    Process(RecordId),
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
            database: args.database,
            extractor: args.extractor,
            property_actor: args.property_actor,
        })
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            MLQueueMsg::Process(id) => {
                if let Err(e) = process_one(state, &id).await {
                    error!(?e, record=%id, "processing ML extraction");
                }
            }
        }
        Ok(())
    }
}

async fn process_one(state: &mut MLQueueState, id: &RecordId) -> Result<(), Whatever> {
    // Load minimal fields needed
    let mut resp = state
        .database
        .query("SELECT title, description FROM $id")
        .bind(("id", id.clone()))
        .await
        .whatever_context("Querying item for ML extraction")?;
    let title: Option<String> = resp
        .take((0, "title"))
        .whatever_context("Taking title from response")?;
    let description: Option<String> = resp
        .take((0, "description"))
        .whatever_context("Taking description from response")?;
    let (Some(title), Some(description)) = (title, description) else {
        debug!(record=%id, "No item found or missing fields for ML extraction");
        return Ok(());
    };

    // Call the extractor via RPC using ractor::call! macro
    match call!(state.extractor, |reply| ExtractionMsg::Process {
        title,
        description,
        rpc_reply_port: reply
    }) {
        Ok(Ok(props)) => {
            info!(record=%id, props=?props, "ML extraction completed");

            for (class, value) in props
                .genres
                .into_iter()
                .map(|v| (Class::Genre, v))
                .chain(props.themes.into_iter().map(|v| (Class::Theme, v)))
                .chain(props.types.into_iter().map(|v| (Class::Type, v)))
                .chain(props.features.into_iter().map(|v| (Class::Feature, v)))
            {
                match call!(state.property_actor, |reply| PropertiesMsg::NewProperty(
                    NewProperty {
                        workshop_item: id.key().to_string(),
                        class: class.clone(),
                        value: value.clone(),
                        note: None,
                    },
                    Source::System,
                    reply
                )) {
                    Ok(Ok(..)) => {
                        debug!(%class, %value, "Inserted new property");
                    }
                    Ok(Err(error)) => {
                        error!(?error,%class, %value,  "Inserting new property");
                    }
                    Err(_) => (),
                }
            }
        }
        Ok(Err(err)) => {
            error!(record=%id, ?err, "ML extraction failed");
        }
        Err(err) => {
            error!(record=%id, ?err, "ML extractor RPC failed");
        }
    }
    Ok(())
}
