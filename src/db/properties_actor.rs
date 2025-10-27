use std::sync::OnceLock;

use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort, async_trait};
use surrealdb::{Surreal, engine::local::Db};

use crate::{
    application::properties_service::PropertiesService,
    db::{model::Source, properties_repository::PropertiesSilo},
    domain::properties::{NewProperty, PropertiesError, VoteData},
};
pub static PROPERTIES_ACTOR: OnceLock<ActorRef<PropertiesMsg>> = OnceLock::new();

/// Actor responsible for handling workshop item properties operations
/// by delegating to the hexagonal `PropertiesService`.
pub struct PropertiesActor;

/// Actor initialization arguments.
pub struct PropertiesArgs {
    pub database: Surreal<Db>,
}

/// Internal state for the actor. Holds the service instance.
pub struct PropertiesState {
    service: PropertiesService<PropertiesSilo>,
}

/// Messages handled by `PropertiesActor`.
pub enum PropertiesMsg {
    NewProperty(
        NewProperty,
        Source<String>,
        RpcReplyPort<Result<(), PropertiesError>>,
    ),
    Vote(VoteData, String, RpcReplyPort<Result<(), PropertiesError>>),
    Remove(VoteData, String, RpcReplyPort<Result<(), PropertiesError>>),
}

#[async_trait]
impl Actor for PropertiesActor {
    type Arguments = PropertiesArgs;
    type Msg = PropertiesMsg;
    type State = PropertiesState;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        PROPERTIES_ACTOR.get_or_init(|| myself);
        Ok(PropertiesState {
            service: PropertiesService::new(PropertiesSilo::new(args.database)),
        })
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            PropertiesMsg::NewProperty(prop, source, reply) => {
                let res = state.service.new_property(prop, source).await;
                let _ = reply.send(res);
            }
            PropertiesMsg::Vote(vote, userid, reply) => {
                let res = state.service.vote(vote, userid).await;
                let _ = reply.send(res);
            }
            PropertiesMsg::Remove(vote, userid, reply) => {
                let res = state.service.remove_vote(vote, userid).await;
                let _ = reply.send(res);
            }
        }
        Ok(())
    }
}
