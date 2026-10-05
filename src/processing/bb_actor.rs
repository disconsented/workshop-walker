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
        // ToDo: Setup bb for the following:

        // [hr][/hr]
        // [table]
        // [cap]
        // let mut config = BBCodeTagConfig::default(); // Default does not
        // include extended tags fyi let mut matchers: Vec<MatchInfo> =
        // vec![]; // A list of NEW matchers we'll pass to from_config
        //
        // // You define how your tag gets turned into HTML using a closure; you
        // are provided the open tag // regex capture, the pre-parsed
        // pre-escaped body, and the closing tag regex capture (if the user
        // provided it). // "EmitScope" is just a fancy alias so you
        // don't have to fuss with the complicated types
        // let color_emitter : EmitScope = Arc::new(|open_capture,body,_c| {
        //     //NOTE: in production code, don't `unwrap` the named capture
        // group, it might not exist!     let color =
        // open_capture.unwrap().name("attr").unwrap().as_str();
        //     format!(r#"<span style="color:{}">{}</span>"#, color, body)
        // });
        //
        // BBCode::add_tagmatcher(&mut matchers, "color",
        // ScopeInfo::basic(color_emitter), None, None)?; //Repeat the
        // emitter / add_tagmatcher above for each tag you want to add

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
