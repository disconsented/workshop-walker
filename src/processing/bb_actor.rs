use bbscope::{BBCode, BBCodeTagConfig};
use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort, async_trait};

pub struct BBActor {}

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
                reply.send(state.bb.parse(&data))?;
            }
        }

        Ok(())
    }
}
