use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort, async_trait};
use tokio::fs::read_to_string;
use tracing::{debug, instrument};

use crate::{Error, MLProperties, populate_prompt, runner::PipelineRunner, sanitise_output};

pub struct ExtractionActor;

pub struct ExtractionArgs;
pub struct ExtractionState {
    pipeline_runner: PipelineRunner,
    features_prompt: String,
    genres_prompt: String,
}

impl ExtractionState {
    #[instrument(skip_all)]
    async fn run_pipeline(
        &self,
        title: String,
        description: String,
    ) -> Result<MLProperties, Error> {
        debug!(title, "running pipeline");
        let features: MLProperties = {
            let features_prompt = populate_prompt(&self.features_prompt, &title, &description);
            let pipeline_response = sanitise_output(
                self.pipeline_runner
                    .run(features_prompt)
                    .await
                    .map_err(|e| Error::Pipeline { source: e })?,
            );
            debug!(pipeline_response, "model returned for features prompt");
            serde_json::from_str(&pipeline_response)?
        };

        let mut genres: MLProperties = {
            let genres_prompt = populate_prompt(&self.genres_prompt, &title, &description);
            let pipeline_response = sanitise_output(
                self.pipeline_runner
                    .run(genres_prompt)
                    .await
                    .map_err(|e| Error::Pipeline { source: e })?,
            );
            debug!(pipeline_response, "model returned for genres prompt");
            serde_json::from_str(&pipeline_response)?
        };

        genres.features.extend(features.features);
        genres.types.extend(features.types);
        genres.genres.extend(features.genres);
        genres.themes.extend(features.themes);
        debug!(ml_properties = ?genres, "pipeline results");
        Ok(genres)
    }
}

pub enum ExtractionMsg {
    Process {
        title: String,
        description: String,
        rpc_reply_port: RpcReplyPort<Result<MLProperties, Error>>,
    },
}
#[async_trait]
impl Actor for ExtractionActor {
    type Arguments = ExtractionArgs;
    type Msg = ExtractionMsg;
    type State = ExtractionState;

    async fn pre_start(
        &self,
        _: ActorRef<Self::Msg>,
        _: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let pipeline_runner = PipelineRunner::setup().await?;

        let features_prompt = read_to_string("./prompts/features.txt").await?;
        let genres_prompt = read_to_string("./prompts/genres.txt").await?;
        Ok(Self::State {
            pipeline_runner,
            features_prompt,
            genres_prompt,
        })
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            ExtractionMsg::Process {
                title,
                description,
                rpc_reply_port,
            } => {
                let _ = rpc_reply_port.send(state.run_pipeline(title, description).await);
            }
        }

        Ok(())
    }
}
