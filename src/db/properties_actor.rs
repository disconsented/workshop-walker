use std::sync::OnceLock;

use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort, async_trait};
use surrealdb::{Surreal, engine::local::Db};
use tracing::{debug, error};

use crate::{
    application::{items_service::ItemsService, properties_service::PropertiesService},
    db::{
        IUserID,
        items_repository::ItemsSilo,
        model::{InternalSource, Property, Status},
        properties_repository::PropertiesSilo,
    },
    domain::properties::{InternalNewProperty, InternalVoteData, PropertiesError},
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
    properties_service: PropertiesService<PropertiesSilo>,
    items_service: ItemsService<ItemsSilo>,
}

/// Messages handled by `PropertiesActor`.
pub enum PropertiesMsg {
    NewSystemProperty(
        InternalNewProperty,
        RpcReplyPort<Result<(), PropertiesError>>,
    ),
    NewUserProperty(
        InternalNewProperty,
        InternalSource,
        RpcReplyPort<Result<(), PropertiesError>>,
    ),
    Vote(
        InternalVoteData,
        IUserID,
        RpcReplyPort<Result<(), PropertiesError>>,
    ),
    Remove(
        InternalVoteData,
        IUserID,
        RpcReplyPort<Result<(), PropertiesError>>,
    ),
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
            properties_service: PropertiesService::new(PropertiesSilo::new(args.database.clone())),
            items_service: ItemsService::new(ItemsSilo::new(args.database)),
        })
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            PropertiesMsg::NewUserProperty(prop, source, reply) => {
                let res = state
                    .properties_service
                    .new_property(prop, source, Status::Pending)
                    .await;
                let _ = reply.send(res);
            }
            PropertiesMsg::NewSystemProperty(prop, reply) => {
                let res = Self::new_system_property(state, prop).await;
                let _ = reply.send(res);
            }
            PropertiesMsg::Vote(vote, userid, reply) => {
                let res = state.properties_service.vote(vote, userid).await;
                let _ = reply.send(res);
            }
            PropertiesMsg::Remove(vote, userid, reply) => {
                let res = state.properties_service.remove_vote(vote, userid).await;
                let _ = reply.send(res);
            }
        }
        Ok(())
    }
}

impl PropertiesActor {
    async fn new_system_property(
        state: &mut PropertiesState,
        prop: InternalNewProperty,
    ) -> Result<(), PropertiesError> {
        debug!(?prop, "new system property");
        let item_id = prop.workshop_item.clone();
        let app = state
            .items_service
            .get_item(item_id.clone())
            .await
            .inspect_err(|error| error!(?error, "getting item for new system property"))
            .map_err(|_| PropertiesError::Internal)?
            .app;
        let (hidden_properties, total_count) = state
            .properties_service
            .get_hidden_system_props(
                Property {
                    class: prop.class.clone(),
                    value: prop.value.clone(),
                }
                .into(),
                app.clone(),
            )
            .await
            .inspect_err(|error| error!(?error, "getting hidden system properties"))?;
        debug!(
            ?hidden_properties,
            ?total_count,
            "getting total system properties"
        );
        let status = if total_count >= 1 {
            Status::Accepted
        } else {
            Status::Hidden
        };
        state
            .properties_service
            .new_property(prop, InternalSource::System, status)
            .await
            .inspect_err(|error| {
                if error != &PropertiesError::Conflict {
                    error!(?error, "creating properties");
                }
            })?;

        if !hidden_properties.is_empty() && status == Status::Accepted {
            state
                .items_service
                .set_property_status(
                    app,
                    hidden_properties.into_iter().collect(),
                    Status::Accepted,
                    Some(Status::Hidden),
                )
                .await
                .inspect_err(|error| error!(?error, "set property status for existing props"))
                .map_err(|_| PropertiesError::Internal)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use surrealdb::{Surreal, engine::local::Db};

    use super::{PropertiesActor, PropertiesState};
    use crate::{
        application::{items_service::ItemsService, properties_service::PropertiesService},
        db::{
            IItemID,
            items_repository::{ItemsSilo, test::seed_db},
            model::Class,
            properties_repository::PropertiesSilo,
        },
        domain::properties::InternalNewProperty,
    };

    /// The classifier output that gives `Feature/shared` to `item`.
    fn shared_feature(item: i64) -> InternalNewProperty {
        InternalNewProperty {
            workshop_item: IItemID::from(item),
            class: Class::Feature,
            value: "shared".to_string(),
            note: None,
        }
    }

    /// The status of each `Feature/shared` link, in item order.
    async fn shared_statuses(db: &Surreal<Db>) -> Vec<i64> {
        let mut result = db
            .query(
                "SELECT VALUE status FROM workshop_item_properties WHERE out = properties:{ \
                 class: 'Feature', value: 'shared' } ORDER BY in;",
            )
            .await
            .unwrap();
        result.take(0).unwrap()
    }

    /// Items 100 and 200 are both in app 1. The first `Feature/shared` link is
    /// a one off, so it stays hidden. The second link makes two, so both
    /// links must show.
    #[tokio::test]
    async fn second_system_property_accepts_both_links() {
        let subscriber = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_line_number(true)
            .with_file(true)
            .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
            .finish();
        tracing::subscriber::set_global_default(subscriber).unwrap();

        let db = seed_db().await;
        db.query(
            "DEFINE FIELD OVERWRITE status ON workshop_item_properties TYPE -2 | -1 | 0 | 1 \
             DEFAULT 0 PERMISSIONS FULL;",
        )
        .await
        .unwrap()
        .check()
        .unwrap();
        let mut state = PropertiesState {
            properties_service: PropertiesService::new(PropertiesSilo::new(db.clone())),
            items_service: ItemsService::new(ItemsSilo::new(db.clone())),
        };

        PropertiesActor::new_system_property(&mut state, shared_feature(100))
            .await
            .expect("the first link should be created");
        assert_eq!(
            shared_statuses(&db).await,
            vec![-2],
            "a one off stays hidden"
        );

        PropertiesActor::new_system_property(&mut state, shared_feature(200))
            .await
            .expect("the second link should be created");
        assert_eq!(
            shared_statuses(&db).await,
            vec![1, 1],
            "two links make the property common, so both show"
        );
    }
}
