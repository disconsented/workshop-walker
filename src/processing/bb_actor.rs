use bbscope::{BBCode, BBCodeTagConfig};
use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort, async_trait};
use tracing::debug_span;

pub struct BBActor {}

#[derive(Debug)]
pub struct BBArgs {}
pub struct BBState {
    bb: BBCode,
}

pub enum BBMsg {
    Process(String, RpcReplyPort<String>),
}
#[async_trait]
impl Actor for BBActor {
    type Arguments = BBArgs;
    type Msg = BBMsg;
    type State = BBState;

    #[tracing::instrument(level = "debug", skip_all)]
    async fn pre_start(
        &self,
        _: ActorRef<Self::Msg>,
        _: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(Self::State {
            bb: BBCode::from_config(BBCodeTagConfig::extended(), None)?,
        })
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            BBMsg::Process(data, reply) => {
                // Nothing awaits here, so entering the span is safe.
                let _entered = debug_span!("bb parse", bytes = data.len()).entered();
                reply.send(state.bb.parse(&data))?;
            }
        }

        Ok(())
    }
}
