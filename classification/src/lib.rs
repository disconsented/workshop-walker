#[cfg(feature = "mkl")]
extern crate intel_mkl_src;

#[cfg(feature = "accelerate")]
extern crate accelerate_src;

use anyhow::{Error as E, Result};
use candle_core::{DType, Device, Module, Tensor};
use candle_examples::token_output_stream::TokenOutputStream;
use candle_nn::VarBuilder;
use candle_transformers::{
    generation::{LogitsProcessor, Sampling},
    models::{
        mimi::candle,
        mistral::{Config, Model as Mistral},
        quantized_mistral::Model as QMistral,
    },
};
use clap::Parser;
use hf_hub::{Repo, RepoType, api::sync::Api};
use serde::Deserialize;
use tokenizers::Tokenizer;

enum Model {
    Mistral(Mistral),
    Quantized(QMistral),
}

struct TextGeneration {
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
            let temperature = temp.unwrap_or(0.);
            let sampling = if temperature <= 0. {
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
            tokenizer: TokenOutputStream::new(tokenizer),
            logits_processor,
            repeat_penalty,
            repeat_last_n,
            device: device.clone(),
        }
    }

    fn reset(&mut self) {
        match &mut self.model {
            Model::Quantized(m) => m.clear_kv_cache(),
            Model::Mistral(m) => m.clear_kv_cache(),
        }
        self.tokenizer.clear();
    }

    fn run(&mut self, prompt: &str, sample_len: usize) -> Result<()> {
        use std::io::Write;
        self.tokenizer.clear();
        let mut tokens = self
            .tokenizer
            .tokenizer()
            .encode(prompt, true)
            .map_err(E::msg)?
            .get_ids()
            .to_vec();
        for &t in tokens.iter() {
            if let Some(t) = self.tokenizer.next_token(t)? {
                print!("{t}")
            }
        }
        std::io::stdout().flush()?;

        let mut generated_tokens = 0usize;
        let eos_token = match self.tokenizer.get_token("</s>") {
            Some(token) => token,
            None => anyhow::bail!("cannot find the </s> token"),
        };
        let start_gen = std::time::Instant::now();
        for index in 0..sample_len {
            let context_size = if index > 0 { 1 } else { tokens.len() };
            let start_pos = tokens.len().saturating_sub(context_size);
            let ctxt = &tokens[start_pos..];
            let input = Tensor::new(ctxt, &self.device)?.unsqueeze(0)?;
            let logits = match &mut self.model {
                Model::Mistral(m) => m.forward(&input, start_pos)?,
                Model::Quantized(m) => m.forward(&input, start_pos)?,
            };
            let logits = logits.squeeze(0)?.squeeze(0)?.to_dtype(DType::F32)?;
            let logits = if self.repeat_penalty == 1. {
                logits
            } else {
                let start_at = tokens.len().saturating_sub(self.repeat_last_n);
                candle_transformers::utils::apply_repeat_penalty(
                    &logits,
                    self.repeat_penalty,
                    &tokens[start_at..],
                )?
            };

            let next_token = self.logits_processor.sample(&logits)?;
            tokens.push(next_token);
            generated_tokens += 1;
            if next_token == eos_token {
                break;
            }
            if let Some(t) = self.tokenizer.next_token(next_token)? {
                print!("{t}");
                std::io::stdout().flush()?;
            }
        }
        let dt = start_gen.elapsed();
        if let Some(rest) = self.tokenizer.decode_rest().map_err(E::msg)? {
            print!("{rest}");
        }
        std::io::stdout().flush()?;
        println!(
            "\n{generated_tokens} tokens generated ({:.2} token/s)",
            generated_tokens as f64 / dt.as_secs_f64(),
        );
        Ok(())
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, clap::ValueEnum, Default)]
enum Which {
    #[default]
    #[value(name = "7b-v0.1")]
    Mistral7bV01,
    #[value(name = "7b-v0.2")]
    Mistral7bV02,
    #[value(name = "7b-instruct-v0.1")]
    Mistral7bInstructV01,
    #[value(name = "7b-instruct-v0.2")]
    Mistral7bInstructV02,
    #[value(name = "7b-instruct-v0.3")]
    Mistral7bInstructV03,
    #[value(name = "7b-maths-v0.1")]
    Mathstral7bV01,
    #[value(name = "nemo-2407")]
    MistralNemo2407,
    #[value(name = "nemo-instruct-2407")]
    MistralNemoInstruct2407,
}

#[derive(Parser, Debug, Default)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run on CPU rather than on GPU.
    #[arg(long)]
    cpu: bool,

    /// Enable tracing (generates a trace-timestamp.json file).
    #[arg(long)]
    tracing: bool,

    #[arg(long)]
    use_flash_attn: bool,

    #[arg(long)]
    prompt: String,

    /// The temperature used to generate samples.
    #[arg(long)]
    temperature: Option<f64>,

    /// Nucleus sampling probability cutoff.
    #[arg(long)]
    top_p: Option<f64>,

    /// Only sample among the top K samples.
    #[arg(long)]
    top_k: Option<usize>,

    /// The seed to use when generating random samples.
    #[arg(long, default_value_t = 299792458)]
    seed: u64,

    /// The length of the sample to generate (in tokens).
    #[arg(long, short = 'n', default_value_t = 10000)]
    sample_len: usize,

    /// The model size to use.
    #[arg(long, default_value = "7b-v0.1")]
    which: Which,

    #[arg(long)]
    model_id: Option<String>,

    #[arg(long, default_value = "main")]
    revision: String,

    #[arg(long)]
    tokenizer_file: Option<String>,

    #[arg(long)]
    config_file: Option<String>,

    #[arg(long)]
    weight_files: Option<String>,

    #[arg(long)]
    quantized: bool,

    /// Penalty to be applied for repeating tokens, 1. means no penalty.
    #[arg(long, default_value_t = 1.1)]
    repeat_penalty: f32,

    /// The context size to consider for the repeat penalty.
    #[arg(long, default_value_t = 64)]
    repeat_last_n: usize,

    /// Use the slower dmmv cuda kernel.
    #[arg(long)]
    force_dmmv: bool,
}

fn run(args: Args) -> Result<()> {
    use tracing_chrome::ChromeLayerBuilder;
    use tracing_subscriber::prelude::*;

    #[cfg(feature = "cuda")]
    candle::quantized::cuda::set_force_dmmv(args.force_dmmv);

    let _guard = if args.tracing {
        let (chrome_layer, guard) = ChromeLayerBuilder::new().build();
        tracing_subscriber::registry().with(chrome_layer).init();
        Some(guard)
    } else {
        None
    };
    println!(
        "avx: {}, neon: {}, simd128: {}, f16c: {}",
        candle::utils::with_avx(),
        candle::utils::with_neon(),
        candle::utils::with_simd128(),
        candle::utils::with_f16c()
    );
    println!(
        "temp: {:.2} repeat-penalty: {:.2} repeat-last-n: {}",
        args.temperature.unwrap_or(0.),
        args.repeat_penalty,
        args.repeat_last_n
    );

    let start = std::time::Instant::now();
    let api = Api::new()?;
    let model_id = match args.model_id {
        Some(model_id) => model_id,
        None => {
            if args.quantized {
                if args.which != Which::Mistral7bV01 {
                    anyhow::bail!("only 7b-v0.1 is available as a quantized model for now")
                }
                "lmz/candle-mistral".to_string()
            } else {
                let name = match args.which {
                    Which::Mistral7bV01 => "mistralai/Mistral-7B-v0.1",
                    Which::Mistral7bV02 => "mistralai/Mistral-7B-v0.3",
                    Which::Mistral7bInstructV01 => "mistralai/Mistral-7B-Instruct-v0.1",
                    Which::Mistral7bInstructV02 => "mistralai/Mistral-7B-Instruct-v0.2",
                    Which::Mistral7bInstructV03 => "mistralai/Mistral-7B-Instruct-v0.3",
                    Which::Mathstral7bV01 => "mistralai/mathstral-7B-v0.1",
                    Which::MistralNemo2407 => "mistralai/Mistral-Nemo-Base-2407",
                    Which::MistralNemoInstruct2407 => "mistralai/Mistral-Nemo-Instruct-2407",
                };
                name.to_string()
            }
        }
    };
    let repo = api.repo(Repo::with_revision(
        model_id,
        RepoType::Model,
        args.revision,
    ));
    let tokenizer_filename = match args.tokenizer_file {
        Some(file) => std::path::PathBuf::from(file),
        None => repo.get("tokenizer.json")?,
    };
    let filenames = match args.weight_files {
        Some(files) => files
            .split(',')
            .map(std::path::PathBuf::from)
            .collect::<Vec<_>>(),
        None => {
            if args.quantized {
                vec![repo.get("model-q4k.gguf")?]
            } else {
                candle_examples::hub_load_safetensors(&repo, "model.safetensors.index.json")?
            }
        }
    };
    println!("retrieved the files in {:?}", start.elapsed());
    let tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(E::msg)?;

    let start = std::time::Instant::now();
    let config = match args.config_file {
        Some(config_file) => serde_json::from_slice(&std::fs::read(config_file)?)?,
        None => {
            if args.quantized {
                Config::config_7b_v0_1(args.use_flash_attn)
            } else {
                let config_file = repo.get("config.json")?;
                serde_json::from_slice(&std::fs::read(config_file)?)?
            }
        }
    };
    let device = candle_examples::device(args.cpu)?;
    let (model, device) = if args.quantized {
        let filename = &filenames[0];
        let vb =
            candle_transformers::quantized_var_builder::VarBuilder::from_gguf(filename, &device)?;
        let model = QMistral::new(&config, vb)?;
        (Model::Quantized(model), device)
    } else {
        let dtype = if device.is_cuda() {
            DType::BF16
        } else {
            DType::F32
        };
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&filenames, dtype, &device)? };
        let model = Mistral::new(&config, vb)?;
        (Model::Mistral(model), device)
    };

    println!("loaded the model in {:?}", start.elapsed());

    let mut pipeline = TextGeneration::new(
        model,
        tokenizer,
        args.seed,
        args.temperature,
        args.top_p,
        args.top_k,
        args.repeat_penalty,
        args.repeat_last_n,
        &device,
    );
    println!("{:?}", std::env::current_dir());
    let prompt1 = std::fs::read_to_string("prompt1.txt")?;
    let prompt2 = std::fs::read_to_string("prompt2.txt")?;
    for i in 0..=4 {
        let prompt1 = prompt1.clone();
        let data: Data =
            serde_json::from_str(&std::fs::read_to_string(format!("test_data/data{i}.json"))?)?;
        let prompt1 = prompt1.replace("[GAME]", &data.game);
        let prompt1 = prompt1.replace("[TITLE]", &data.title);
        let prompt1 = prompt1.replace("[DESCRIPTION]", &data.description);
        pipeline.run(&prompt1, args.sample_len)?;
        pipeline.reset();
        let prompt2 = prompt2.clone();
        let prompt2 = prompt2.replace("[GAME]", &data.game);
        let prompt2 = prompt2.replace("[TITLE]", &data.title);
        let prompt2 = prompt2.replace("[DESCRIPTION]", &data.description);
        pipeline.run(&prompt2, args.sample_len)?;
        pipeline.reset();
    }
    Ok(())
}
// fn main() -> Result<()> {
//     let args = Args::parse();
//     run(args)
// }

#[derive(Deserialize)]
struct Data {
    game: String,
    title: String,
    description: String,
}

#[cfg(test)]
mod test {
    use crate::{Args, Which, run};

    // Baseline test, should be ran with the release profile. Probably with
    // cranelift in the future
    #[test]
    fn test_prompts() {
        let args = Args {
            sample_len: 5000,
            which: Which::Mistral7bInstructV03,
            prompt: String::new(), // Ignored
            revision: String::from("main"),
            ..Default::default()
        };

        run(args).unwrap();
    }

    // Alpha Animals:
    //  {
    //   "types": ["addon", "expansion"],
    //   "genres": [],
    //   "themes": [],
    //   "compatible_items": ["VFE Insectoids 2", "VRE Insectors", "Alpha
    // Biomes"] }
    //  {
    //   "features": [
    //     "new creatures",
    //     "unique animals",
    //     "vanilla-friendly creatures",
    //     "new mechanic",
    //     "walking tanks",
    //     "living resource farm",
    //     "indestructible plant monsters",
    //     "night time stalkers",
    //     "giant spiders",
    //     "new alien biomes",
    //     "black hive faction",
    //     "new insects",
    //     "black hive raids (mod options)",
    //     "new animals (future plans)"
    //   ]
    // }
    // DMS:
    // {
    // "types": ["mod", "expansion", "graphics", "texture", "audio", "ui"],
    // "genres": ["cyberpunk"],
    // "themes": ["science fiction", "mecha"],
    // "compatible_items": []
    // }
    // {
    // "features": [
    // "heavy metallic weapons",
    // "industrial-tactical robots",
    // "bio-mech bionic",
    // "new scenario",
    // "standalone technology tree",
    // "ideology style pack",
    // "interstellar colonization company",
    // "royalty title system",
    // "military-industrial-style killing machines",
    // "customizable mechanoid weapons",
    // "new clothes",
    // "CE compatibility"
    // ]
    // }
    // VFE:
    // {
    // "types": ["mod", "graphics", "ui"],
    // "genres": [],
    // "themes": [],
    // "compatible_items": []
    // }
    // {
    // "features": [
    // "vehicle framework for modders",
    // "equal treatment for land, sea, aerial vehicles",
    // "vehicle health system",
    // "external and internal components",
    // "physical hitboxes on vehicles",
    // "custom pathfinding for vehicles",
    // "multi-cell pathfinding",
    // "3 shaders for patterns and skins",
    // "turrets with custom positions",
    // "aerial vehicles with separate skyfaller launching system",
    // "animation editor for vehicles",
    // "graphic overlay system",
    // "mod settings page",
    // "c# attribute for modder's ThingComps",
    // "long-term features: raiders and traders use vehicles, aerial vehicle
    // events, air defense, joint warfare, combined arms tactics" ]
    // }
    // A Rimworld of Magic
    //  {
    //   "types": ["mod", "expansion", "graphics", "texture", "audio", "ui",
    // "bugfix", "patch"],   "genres": ["fantasy"],
    //   "themes": ["magic", "role-playing"],
    //   "compatible_items": []
    // }
    //  {
    //   "features": [
    //     "unique classes",
    //     "unique abilities",
    //     "unique development tree",
    //     "unique apparel/equipment",
    //     "gem crafting",
    //     "enchantments",
    //     "magical buildings",
    //     "new events",
    //     "unique research tree",
    //     "magic faction",
    //     "ai classes with abilities",
    //     "mod options",
    //     "unique power balance"
    //   ]
    // }
    // Save Our Ship 2
    // {
    // "types": ["expansion", "mod"],
    // "genres": [],
    // "themes": ["space", "scifi", "shipbuilding", "combat", "adventure"],
    // "compatible_items": ["Harmony", "Vehicle Framework"]
    // }
    // {
    // "features": [
    // "custom endings",
    // "ship-to-ship combat",
    // "derelict searching",
    // "cosmic secret discovery",
    // "machine godhood evolution",
    // "new features",
    // "stability upgrades",
    // "ship building",
    // "pre-flight checklist",
    // "ship lifting off",
    // "vanilla expanded textures",
    // "heat system overhaul",
    // "shields",
    // "reactors",
    // "weapons",
    // "travel to new world",
    // "new game plus",
    // "final end game quest",
    // "shuttle upgrades",
    // "mod options",
    // "ship physics",
    // "boarding party hard mode",
    // "Save Our Ship Wiki",
    // "community wiki volunteering",
    // "ship creation kit",
    // "workshop sharing",
    // "compatibility sheet",
    // "mod compatibility reporting",
    // "developed by Kentington and Thain",
    // "testers",
    // "submod support",
    // "contributors",
    // "shipwrights",
    // "OG Testing Squad"
    // ]
    // }
}
