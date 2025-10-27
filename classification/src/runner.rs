use candle_core::{DType, Device};
use candle_nn::VarBuilder;
use candle_transformers::models::mistral::Model;
use hf_hub::{Repo, RepoType, api::tokio::Api};
use snafu::ResultExt;
use tokenizers::Tokenizer;
use tokio::{
    sync::{mpsc, oneshot},
    task::{JoinHandle, spawn_blocking},
};
use tracing::warn;

use crate::{
    Error, MODEL_ID, ModelInitSnafu, ParseConfigSnafu, ReadConfigSnafu, TextGeneration,
    TokenizerLoadSnafu, VarBuilderLoadSnafu, WhateverAsync, hub::hub_load_safetensors,
};

pub struct PipelineRunner {
    pipeline_task: JoinHandle<()>,
    pipeline_tx: mpsc::Sender<(String, oneshot::Sender<Result<String, WhateverAsync>>)>,
}
impl PipelineRunner {
    pub async fn setup() -> Result<Self, Error> {
        let api = Api::new().map_err(|e| Error::ApiInit {
            message: e.to_string(),
        })?;
        let revision = "main".to_string();
        let repo = api.repo(Repo::with_revision(
            MODEL_ID.to_string(),
            RepoType::Model,
            revision,
        ));
        let filenames = hub_load_safetensors(&repo, "model.safetensors.index.json")
            .await
            .map_err(|e| Error::HubLoadIndex {
                message: e.to_string(),
            })?;
        let tokenizer_filename = repo
            .get("tokenizer.json")
            .await
            .map_err(|e| Error::RepoGet {
                filename: "tokenizer.json",
                message: e.to_string(),
            })?;
        let device = Device::Cpu;
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&filenames, DType::F32, &device)
                .context(VarBuilderLoadSnafu)?
        };
        let tokenizer = Tokenizer::from_file(tokenizer_filename).context(TokenizerLoadSnafu)?;
        let config_file = repo.get("config.json").await.map_err(|e| Error::RepoGet {
            filename: "config.json",
            message: e.to_string(),
        })?;
        let config = serde_json::from_slice(&std::fs::read(config_file).context(ReadConfigSnafu)?)
            .context(ParseConfigSnafu)?;
        let model = Model::new(&config, vb).context(ModelInitSnafu)?;
        let (pipeline_tx, mut pipeline_rx) =
            mpsc::channel::<(String, oneshot::Sender<Result<String, WhateverAsync>>)>(1);
        let pipeline_task: JoinHandle<()> = spawn_blocking(move || {
            let mut pipeline = TextGeneration::new(
                model, tokenizer, 299792458, None, None, None, 1.1, 64, &device,
            );

            while let Some((task, reply)) = pipeline_rx.blocking_recv() {
                let _ = reply.send(pipeline.run(task.as_str(), 100_000));
            }

            warn!("Channel has dropped; exiting");
        });

        Ok(Self {
            pipeline_task,
            pipeline_tx,
        })
    }

    pub async fn run(&self, prompt: String) -> Result<String, WhateverAsync> {
        let (tx, rx) = oneshot::channel();
        self.pipeline_tx
            .send((prompt, tx))
            .await
            .whatever_context("sending to tasks runner")?;
        rx.await.whatever_context("receiving response")?
    }
}

impl Drop for PipelineRunner {
    fn drop(&mut self) {
        self.pipeline_task.abort();
    }
}
