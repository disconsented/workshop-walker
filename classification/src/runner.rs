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
use tracing::{Instrument, info_span, warn};

use crate::{
    Error, MODEL_ID, ParseConfigSnafu, ReadConfigSnafu, TextGeneration, TokenizerLoadSnafu,
    VarBuilderLoadSnafu, WhateverAsync, hub::hub_load_safetensors,
};

pub struct PipelineRunner {
    pipeline_task: JoinHandle<()>,
    pipeline_tx: mpsc::Sender<(String, oneshot::Sender<Result<String, WhateverAsync>>)>,
}
impl PipelineRunner {
    pub async fn setup() -> Result<Self, Error> {
        let span = info_span!("PipelineRunner::setup");
        let _g = span.enter();
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
            .instrument(info_span!(parent: &span, "load safe tensors"))
            .await
            .map_err(|e| Error::HubLoadIndex {
                message: e.to_string(),
            })?;
        let tokenizer_filename = repo
            .get("tokenizer.json")
            .instrument(info_span!(parent: &span, "load tokenizer"))
            .await
            .map_err(|e| Error::RepoGet {
                filename: "tokenizer.json",
                message: e.to_string(),
            })?;
        let device = Device::Cpu;
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&filenames, DType::F16, &device)
                .context(VarBuilderLoadSnafu)?
        };
        let tokenizer = Tokenizer::from_file(tokenizer_filename).context(TokenizerLoadSnafu)?;
        let config_file = repo
            .get("config.json")
            .instrument(info_span!(parent: &span, "get config"))
            .await
            .map_err(|e| Error::RepoGet {
                filename: "config.json",
                message: e.to_string(),
            })?;
        let file_bytes = tokio::fs::read(config_file)
            .instrument(info_span!(parent: &span, "read config"))
            .await
            .context(ReadConfigSnafu)?;
        let config = info_span!(parent: &span, "parse config")
            .in_scope(|| serde_json::from_slice(&file_bytes))
            .context(ParseConfigSnafu)?;
        let (pipeline_tx, mut pipeline_rx) =
            mpsc::channel::<(String, oneshot::Sender<Result<String, WhateverAsync>>)>(1);
        let pipeline_task: JoinHandle<()> = spawn_blocking(move || {
            let model = info_span!("Model setup").in_scope(|| Model::new(&config, vb).unwrap());
            let mut pipeline = TextGeneration::new(
                model, tokenizer, 299792458, None, None, None, 1.1, 64, &device,
            );

            while let Some((task, reply)) = pipeline_rx.blocking_recv() {
                let _ = info_span!("run model")
                    .in_scope(|| reply.send(pipeline.run(task.as_str(), 100_000)));
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
