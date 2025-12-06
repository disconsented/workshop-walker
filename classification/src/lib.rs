#[cfg(feature = "mkl")]
extern crate intel_mkl_src;
pub mod actor;
mod hub;
mod runner;

use std::backtrace::Backtrace;

use snafu::{ResultExt, Snafu, whatever};

const MODEL_ID: &str = "mistralai/Mistral-7B-Instruct-v0.2";

use candle_core::{DType, Device, Tensor};
use candle_examples::token_output_stream::TokenOutputStream;
use candle_transformers::{
    generation::{LogitsProcessor, Sampling},
    models::mistral::Model,
};
use serde::{Deserialize, Serialize};
use tokenizers::Tokenizer;
use tracing::{debug, instrument};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("failed to initialise HF Hub API: {message}"))]
    ApiInit { message: String },

    #[snafu(display("failed to load model index from hub: {message}"))]
    HubLoadIndex { message: String },

    #[snafu(display("failed to get '{filename}' from hub: {message}"))]
    RepoGet {
        filename: &'static str,
        message: String,
    },

    #[snafu(display("failed to create VarBuilder from safetensors: {source}"))]
    VarBuilderLoad { source: candle_core::Error },

    #[snafu(display("failed to load tokenizer: {source}"))]
    TokenizerLoad { source: tokenizers::Error },

    #[snafu(display("tokenizer encode failed: {source}"))]
    TokenizerEncode { source: tokenizers::Error },

    #[snafu(display("missing required token: {token}"))]
    MissingToken { token: &'static str },

    #[snafu(display("tensor operation failed: {source}"))]
    TensorOp { source: candle_core::Error },

    #[snafu(display("model forward failed: {source}"))]
    Forward { source: candle_core::Error },

    #[snafu(display("sampling next token failed: {source}"))]
    Sample { source: candle_core::Error },

    #[snafu(display("repeat penalty application failed: {source}"))]
    RepeatPenalty { source: candle_core::Error },

    #[snafu(display("token stream operation failed: {source}"))]
    TokenStream {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[snafu(display("failed to read config: {source}"))]
    ReadConfig { source: std::io::Error },

    #[snafu(display("failed to parse config.json: {source}"))]
    ParseConfig { source: serde_json::Error },

    #[snafu(display("failed to build model: {source}"))]
    ModelInit { source: candle_core::Error },

    // New errors used by classification::actor
    #[snafu(display("task join failed: {source}"))]
    Join { source: tokio::task::JoinError },

    #[snafu(display("failed to parse model output JSON: {source}"))]
    ParseResult { source: serde_json::Error },

    #[snafu(display("pipeline failed: {source}"))]
    Pipeline { source: WhateverAsync },
}

// https://github.com/shepmaster/snafu/pull/448#issuecomment-3554318299
#[derive(Debug, Snafu)]
#[snafu(whatever)]
#[snafu(display("{message}"))]
#[snafu(provide(opt, ref, chain, dyn std::error::Error + Send + Sync => source.as_deref()))]
pub struct WhateverAsync {
    #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, Some)))]
    #[snafu(provide(false))]
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
    message: String,
    backtrace: Backtrace,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl From<tokio::task::JoinError> for Error {
    fn from(source: tokio::task::JoinError) -> Self {
        Self::Join { source }
    }
}

impl From<serde_json::Error> for Error {
    fn from(source: serde_json::Error) -> Self {
        Self::ParseResult { source }
    }
}

pub struct TextGeneration {
    model: Model,
    device: Device,
    tokenizer: TokenOutputStream,
    logits_processor: LogitsProcessor,
    repeat_penalty: f32,
    repeat_last_n: usize,
}

impl TextGeneration {
    #[allow(clippy::too_many_arguments)]
    fn new(
        model: Model,
        tokenizer: Tokenizer,
        seed: u64,
        temp: Option<f64>,
        top_p: Option<f64>,
        top_k: Option<usize>,
        repeat_penalty: f32,
        repeat_last_n: usize,
        device: &Device,
    ) -> Self {
        let logits_processor = {
            let temperature = temp.unwrap_or(0.0);
            let sampling = if temperature <= 0.0 {
                Sampling::ArgMax
            } else {
                match (top_k, top_p) {
                    (None, None) => Sampling::All { temperature },
                    (Some(k), None) => Sampling::TopK { k, temperature },
                    (None, Some(p)) => Sampling::TopP { p, temperature },
                    (Some(k), Some(p)) => Sampling::TopKThenTopP { k, p, temperature },
                }
            };
            LogitsProcessor::from_sampling(seed, sampling)
        };

        Self {
            model,
            logits_processor,
            repeat_penalty,
            repeat_last_n,
            device: device.clone(),
            tokenizer: TokenOutputStream::new(tokenizer),
        }
    }

    #[instrument(skip_all)]
    fn run(&mut self, prompt: &str, sample_len: usize) -> Result<String, WhateverAsync> {
        let encoding = self
            .tokenizer
            .tokenizer()
            .encode(prompt, true)
            .map_err(|e| e.to_string())
            .whatever_context("encode prompt")?;
        self.model.clear_kv_cache();
        let mut tokens = encoding.get_ids().to_vec();
        let mut input_tokens = String::new();
        for &t in tokens.iter() {
            if let Some(t) = self.tokenizer.next_token(t).unwrap() {
                input_tokens += &t;
            }
        }

        let eos_token = match self.tokenizer.get_token("</s>") {
            Some(token) => token,
            None => whatever!("cannot find the </s> token"),
        };
        self.tokenizer.clear();
        let mut output = String::new();
        for index in 0..sample_len {
            let context_size = if index > 0 { 1 } else { tokens.len() };
            let start_pos = tokens.len().saturating_sub(context_size);
            let ctxt = &tokens[start_pos..];
            let input = Tensor::new(ctxt, &self.device)
                .whatever_context("create input tensor")?
                .unsqueeze(0)
                .whatever_context("unsqueeze batch dim")?;
            let logits = self
                .model
                .forward(&input, start_pos)
                .whatever_context("model forward")?;
            let logits = logits
                .squeeze(0)
                .whatever_context("squeeze time dim")?
                .squeeze(0)
                .whatever_context("squeeze seq dim")?
                .to_dtype(DType::F32)
                .whatever_context("cast logits to f32")?;
            let logits = if self.repeat_penalty == 1. {
                logits
            } else {
                let start_at = tokens.len().saturating_sub(self.repeat_last_n);
                candle_transformers::utils::apply_repeat_penalty(
                    &logits,
                    self.repeat_penalty,
                    &tokens[start_at..],
                )
                .whatever_context("apply_repeat_penalty")?
            };

            let next_token = self
                .logits_processor
                .sample(&logits)
                .whatever_context("sample next token")?;
            tokens.push(next_token);
            if next_token == eos_token {
                break;
            }
            if let Some(t) = self
                .tokenizer
                .next_token(next_token)
                .whatever_context("stream next token")?
            {
                output.push_str(&t);
            }
        }
        if let Some(rest) = self
            .tokenizer
            .decode_rest()
            .whatever_context("decode remaining tokens")?
        {
            output.push_str(&rest);
        }

        debug!(output, "finished running ML");
        Ok(output)
    }
}

/// The text replacement cases were things I ran into during development, the
/// '{' search is an effort to protect against prompt repeating by the model.
pub fn sanitise_output(string: String) -> String {
    let nothing = "";
    let string = &string[string.find('{').unwrap_or_default()..];
    string
        .replace("<br>", nothing)
        .replace("###", nothing)
        .replace(">", nothing)
        .replace("```json", nothing)
        .replace("```", nothing)
        .replace("Output:", nothing)
}

/// This took a lot longer to develop than expected, because, having a tab
/// character at the end would sometimes cause the model to just repeat the end
/// of its prompt.
pub fn populate_prompt(prompt: &str, title: &str, description: &str) -> String {
    prompt
        .replace("[TITLE]", title)
        .replace("[DESCRIPTION]", description)
        .replace("\t", "")
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MLProperties {
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(default)]
    pub themes: Vec<String>,
    #[serde(default)]
    pub types: Vec<String>,
    #[serde(default)]
    pub features: Vec<String>,
}

#[cfg(test)]
mod test {
    use std::fs::read_to_string;

    use candle_core::{DType, Device};
    use candle_examples::hub_load_safetensors;
    use candle_nn::VarBuilder;
    use candle_transformers::models::mistral::Model;
    use hf_hub::{Repo, RepoType, api::sync::Api};
    use snafu::ResultExt;
    use tokenizers::Tokenizer;

    use crate::{
        Error, MLProperties, MODEL_ID, ModelInitSnafu, ParseConfigSnafu, ReadConfigSnafu,
        TextGeneration, TokenizerLoadSnafu, VarBuilderLoadSnafu, populate_prompt, sanitise_output,
    };
    const DMS_T: &str = "The Dead Man's Switch";
    const DMS_D: &str = r#"<a href="https://steamcommunity.com/id/FUJIKENGAWA/myworkshopfiles/?appid=294100&amp;sort=score&amp;browsefilter=myfiles&amp;view=imagewall" target="_blank"><img src="https://i.imgur.com/kvppfWO.png"></a><br><br>Millennia earlier, the threat of archotech war machines, pirates, and awry bio-organic weapons arose. Nara an Interstellar Industries Complex initiated the project of semi-automated war-machines, which aimed at creating a legion of war-machines with low technology and maintenance requirements to aid humans in all galaxies in the war against all those threats.<br><br>It was a quite successful project, and as countless fleets of unknown generations of ships marched toward the borders of human civilization, the production technology of these weapons also spread in the edge world... until today.<br><br><h1>Introduction</h1>This is a large-scale mod that was created around the cyberpunk theme of the 1990s. It has a lot of content, including heavy metallic weapons, industrial-tactical robots, and bio-mech bionic, so I don’t intend to teach you how to play it. You experience the story you want to play with the new choices! I believe you will find the joy you want.<br><br><h1>Content</h1><img src="https://i.imgur.com/Z61a1rv.png"><br>A new Scenario<br>A standalone technology tree<br>An Ideology style pack<br>A well-prepared interstellar colonization company<br>A royalty Title system based on military organization<br>A cargo load of heavy weapons and more than 20 types of military-industrial-style killing machines.<br>Mechanoids that have customizable weapons.<br>Cheap but costly bionic<br>lots of new clothes<br>CE compatibility<br><br>Try it for yourself, I guarantee it&#x27;s worth a try.<br><br><h1>FAQ</h1>Q:how to equip mech weapon? (For mech)<br>A:Select Mech and right-click the weapon, if the weapon is supported it will be available to equip.<br><br>Q:how to equip mech weapon? (For Colonists)<br>A:They need an exo-skelton suit before equip it.<br><br>Q:What weapon did a mech supported<br>A: you can find the supported weapon inside the info card.<br><br>Q: how to get ____ compoment<br>A: defeat boss group,or trading<br><br>Q: can i disable some of mech&#x27;s worktype?<br>A: <a href="https://steamcommunity.com/sharedfiles/filedetails/?id=3268299107" target="_blank">https://steamcommunity.com/sharedfiles/filedetails/?id=3268299107</a><br><h1>Warning - A new game is highly recommended</h1><br><h1>Known Issues</h1>work mech will get error loading with Rebound.<br><br><h1>Author’s words</h1>This is my fourth year modding in the Rimworld community, also my first year of studying for a master&#x27;s degree after college. I apologize to everyone who has been waiting for this mod for a long time (it’s been a while). Making such a big project is well-challenged.<br><br>DMS is a response that condenses my experience and understanding of countless works to the world and those works and creators who have profoundly affected my life.<br>Completing this project, which included my understanding of Rimworld art and those coolish Science Fiction styles, is also the realization of my dream of creating cool mecha works since I was a child.<br>Therefore, it is quite a torment in terms of technology, time, development enthusiasm, and progress management.<br>From the beginning of the project to the release, it spanned two game versions. Countless setbacks and subversions made me doubt my motivation to continue doing it more than once and questioned whether my work and abilities met expectations...<br><br>But this has all passed, and I have reached a reconciliation between my heavy academic workload and my pursuit of excellence. It was unanimously decided to release the mod before the end of 2023. which is today.<br>Although there may still be some minor problems, anyway still hope you enjoy it. _AOBA 2023/12/25<br><br>If you find any problems or have any ideas while playing, you can give feedback to the Discord group<br><a href="https://discord.gg/Pvj5Xj3yBm" target="_blank"><img src="https://i.ibb.co/CHY91mx/discord-Icon.png"></a><br><a href="https://www.paypal.com/paypalme/AobaKuma" target="_blank"><img src="https://i.ibb.co/cLqX4rv/Paypal-Icon.png"></a><br><a href="https://ko-fi.com/aobakuma" target="_blank"><img src="https://i.ibb.co/KhN0Tgp/Ko-Fi-Icon.png"></a><br><br><br>	"#;

    #[test]
    fn test_dms() {
        let themes_prompt = read_to_string("../prompts/features.txt").unwrap();
        let themes_replaced = populate_prompt(&themes_prompt, DMS_T, DMS_D);

        let themes_json = sanitise_output(run(&themes_replaced).unwrap());
        println!("{themes_json}");
        let output: MLProperties = serde_json::from_slice(themes_json.as_bytes()).unwrap();
        dbg!(output);

        let features_prompt = read_to_string("../prompts/genres.txt").unwrap();
        let features_replaced = populate_prompt(&features_prompt, DMS_T, DMS_D);
        println!("{features_replaced:?}");
        let features_json = sanitise_output(run(&features_replaced).unwrap());
        println!("Features:{features_json}");
        let output: MLProperties = serde_json::from_slice(features_json.as_bytes()).unwrap();
        dbg!(output);
    }
    #[test]
    fn test_xeno() {
        let title = r#"Euglena Expanded - Euglena Xenotype (Continued)"#;
        let description = r#"<br>Original mod by DemonRoka<br><a href="https://steamcommunity.com/sharedfiles/filedetails/?id=2975005239" target="_blank">https://steamcommunity.com/sharedfiles/filedetails/?id=2975005239</a><br>If the original author requests it, I will remove this update.<br><br>--<br><br>Original mod notes (1.5):<br><br>Description:<br>	<br>&quot;A unique tree-like race. These pawns are a symbiosis of plant and animal life, capable of photosynthesis and enhancing their characteristics while in the open sun.<br><br>Mod Features:<br><br>A new race of pawns: the Euglena.<br>Pawns of this race can perform photosynthesis, converting sunlight into nutrients.<br>Eugenes are endowed with the ability to regenerate, allowing them to regenerate lost body parts over time.<br>Euglens require light to maintain their health and can do the work of absorbing light on their own to replenish their energy.<br>Discover this unique race and make the most of their abilities!<br><br>Note: The mod &quot;Euglena Expanded - Euglena Xenotype&quot; is compatible with most other mods and requires only the official add-ons, and the Euglena Framework to work. However, we always recommend that you first check the compatibility of mods in a test game.<br><br>Описание:<br><br>&quot;Уникальную древоподобная раса. Эти пешки - симбиоз растительной и животной жизни, способные осуществлять фотосинтез и улучшать свои характеристики находясь под открытым солнцем.<br><br>Особенности мода:<br><br>Новая раса пешек: Евглена.<br>Пешки этой расы могут осуществлять фотосинтез, превращая солнечный свет в питательные вещества.<br>Евглены наделены способностью регенерации, что позволяет им восстанавливать утраченные части тела со временем.<br>Евглены требуют освещения для поддержания своего здоровья и могут самостоятельно выполнять работу по поглощению света для восполнения своей энергии.<br>Откройте для себя эту уникальную расу и используйте их способности по максимуму!<br><br>Примечание: Мод &quot;Euglena Expanded - Euglena Xenotype&quot; совместим с большинством других модов и требует для своей работы лишь официальные дополнения, и Euglena Framework. Однако мы всегда рекомендуем сначала проверить совместимость модов в тестовой игре. <br>	"#;
        let themes_prompt = read_to_string("../prompts/features.txt").unwrap();
        let themes_replaced = populate_prompt(&themes_prompt, title, description);

        let themes_json = sanitise_output(run(&themes_replaced).unwrap());
        println!("{themes_json}");
        let output: MLProperties = serde_json::from_slice(themes_json.as_bytes()).unwrap();
        dbg!(output);

        let features_prompt = read_to_string("../prompts/genres.txt").unwrap();
        let features_replaced = populate_prompt(&features_prompt, title, description);
        println!("{features_replaced:?}");
        let features_json = sanitise_output(run(&features_replaced).unwrap());
        println!("Features:{features_json}");
        let output: MLProperties = serde_json::from_slice(features_json.as_bytes()).unwrap();
        dbg!(output);
    }

    fn run(prompt: &str) -> crate::Result<String> {
        let api = Api::new().map_err(|e| Error::ApiInit {
            message: e.to_string(),
        })?;
        let revision = "main".to_string();
        let repo = api.repo(Repo::with_revision(
            MODEL_ID.to_string(),
            RepoType::Model,
            revision,
        ));
        let filenames =
            hub_load_safetensors(&repo, "model.safetensors.index.json").map_err(|e| {
                Error::HubLoadIndex {
                    message: e.to_string(),
                }
            })?;
        let tokenizer_filename = repo.get("tokenizer.json").map_err(|e| Error::RepoGet {
            filename: "tokenizer.json",
            message: e.to_string(),
        })?;
        let device = Device::Cpu;
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&filenames, DType::F16, &device)
                .context(VarBuilderLoadSnafu)?
        };
        let tokenizer = Tokenizer::from_file(tokenizer_filename).context(TokenizerLoadSnafu)?;
        let config_file = repo.get("config.json").map_err(|e| Error::RepoGet {
            filename: "config.json",
            message: e.to_string(),
        })?;
        let config = serde_json::from_slice(&std::fs::read(config_file).context(ReadConfigSnafu)?)
            .context(ParseConfigSnafu)?;
        let model = Model::new(&config, vb).context(ModelInitSnafu)?;
        let mut pipeline = TextGeneration::new(
            model, tokenizer, 299792458, None, None, None, 1.1, 64, &device,
        );

        Ok(pipeline.run(prompt, 10000).map(sanitise_output).unwrap())
    }
}
