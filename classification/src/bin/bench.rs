//! Resource and throughput benchmark for the extraction pipeline.
//!
//! One generation per process. The process does not reuse the model, because
//! `candle_transformers::models::qwen3_vl` gives no way to clear its KV cache,
//! and because a single run per process makes the peak RSS exact.
//!
//! The binary writes one JSON file per run. It does not score the output; it
//! keeps the raw text so that a person can score it later.

#[cfg(feature = "mkl")]
extern crate intel_mkl_src;

use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use candle_core::{DType, Device, Tensor};
use candle_examples::{chat_template::ChatTemplate, token_output_stream::TokenOutputStream};
use candle_nn::VarBuilder;
use candle_transformers::{
    generation::{LogitsProcessor, Sampling},
    models::{gemma4, mistral, qwen3_vl},
};
use clap::{Parser, ValueEnum};
use hf_hub::{Repo, RepoType, api::tokio::Api};
use serde::{Deserialize, Serialize};
use tokenizers::Tokenizer;

/// Marks the place in the user message where the image tokens go.
const IMAGE_MARKER: &str = "<<IMAGE>>";

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ModelKey {
    Mistral,
    Qwen3Vl,
    Gemma4,
}

impl ModelKey {
    fn default_repo(self) -> &'static str {
        match self {
            Self::Mistral => "mistralai/Mistral-7B-Instruct-v0.3",
            Self::Qwen3Vl => "Qwen/Qwen3-VL-8B-Instruct",
            Self::Gemma4 => "google/gemma-4-E4B-it",
        }
    }

    /// The tokens that end a turn. The pipeline used to look for `</s>` only,
    /// which stops Mistral but never stops Qwen or Gemma.
    fn stop_tokens(self) -> &'static [&'static str] {
        match self {
            Self::Mistral => &["</s>"],
            Self::Qwen3Vl => &["<|im_end|>", "<|endoftext|>"],
            // Gemma 4 renamed its turn markers: `<turn|>` (106) ends a
            // turn, and `<end_of_turn>` is not in the vocabulary at all.
            Self::Gemma4 => &["<turn|>", "<eos>"],
        }
    }

    fn supports_vision(self) -> bool {
        matches!(self, Self::Qwen3Vl | Self::Gemma4)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum, Serialize)]
#[serde(rename_all = "kebab-case")]
enum PromptKey {
    Features,
    Genres,
}

impl PromptKey {
    fn file_name(self) -> &'static str {
        match self {
            Self::Features => "features.txt",
            Self::Genres => "genres.txt",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum, Serialize)]
#[serde(rename_all = "lowercase")]
enum DTypeArg {
    F16,
    Bf16,
    F32,
}

impl From<DTypeArg> for DType {
    fn from(value: DTypeArg) -> Self {
        match value {
            DTypeArg::F16 => Self::F16,
            DTypeArg::Bf16 => Self::BF16,
            DTypeArg::F32 => Self::F32,
        }
    }
}

#[derive(Parser, Debug)]
#[command(about = "Measures memory and throughput for one extraction run")]
struct Args {
    #[arg(long, value_enum)]
    model: ModelKey,

    /// Overrides the hub repository, to pin a revision or to use a local copy.
    #[arg(long)]
    repo: Option<String>,

    /// The workshop item to classify.
    #[arg(long)]
    item: String,

    #[arg(long, value_enum, default_value = "features")]
    prompt: PromptKey,

    /// Adds the preview image to the prompt. Vision models only.
    #[arg(long)]
    image: bool,

    #[arg(long, value_enum, default_value = "f16")]
    dtype: DTypeArg,

    /// Stops generation after this many tokens. The old pipeline passed
    /// 100_000, which turns one missed stop token into an overnight run.
    #[arg(long, default_value_t = 256)]
    max_tokens: usize,

    #[arg(long, default_value = "./prompts")]
    prompt_dir: PathBuf,

    #[arg(long, default_value = "./bench-out")]
    out_dir: PathBuf,

    /// Caches the item JSON and the preview image here.
    #[arg(long, default_value = "./bench-cache")]
    cache_dir: PathBuf,

    #[arg(long, default_value = "https://workshop-walker.disconsented.com")]
    api_base: String,

    /// Matches `runner.rs`, which passes 1.1 over the last 64 tokens.
    #[arg(long, default_value_t = 1.1)]
    repeat_penalty: f32,

    #[arg(long, default_value_t = 64)]
    repeat_last_n: usize,

    /// Labels the run, to tell a cold load from a warm one.
    #[arg(long, default_value = "warm")]
    label: String,

    /// Fetches the weights and stops. Lets a download overlap with a run of
    /// another model instead of waiting for the machine to be free.
    #[arg(long)]
    download_only: bool,
}

#[derive(Debug, Serialize)]
struct RunReport {
    model: ModelKey,
    repo: String,
    item: String,
    title: String,
    prompt: PromptKey,
    with_image: bool,
    dtype: DTypeArg,
    label: String,
    threads: Option<String>,
    repeat_penalty: f32,

    weight_bytes: u64,
    load_secs: f64,
    tokenizer_secs: f64,

    prompt_chars: usize,
    prefill_tokens: usize,
    image_tokens: usize,
    prefill_secs: f64,
    decode_tokens: usize,
    decode_secs: f64,
    decode_tokens_per_sec: f64,
    generate_secs: f64,
    total_secs: f64,

    hit_token_cap: bool,
    stop_token: Option<String>,

    peak_rss_bytes: u64,
    final_rss_bytes: u64,

    /// The model output, untouched. `sanitise_output` strips every `>`, so the
    /// raw text is the only complete record.
    raw_output: String,
    parses_as_json: bool,
}

#[derive(Debug, Deserialize)]
struct Item {
    title: String,
    description: String,
    preview_url: Option<String>,
}

/// The image, already turned into whatever shape the model wants, plus the
/// number of placeholder tokens it needs in the prompt.
enum PreparedImage {
    /// `pixel_values` of (num_patches, patch_dim) and a grid of (1, h, w).
    Qwen { patches: Tensor, grid: Tensor, tokens: usize },
    /// One (1, c, h, w) tensor.
    Gemma { pixels: Tensor, tokens: usize },
}

impl PreparedImage {
    fn tokens(&self) -> usize {
        match self {
            Self::Qwen { tokens, .. } | Self::Gemma { tokens, .. } => *tokens,
        }
    }
}

enum Backend {
    Mistral(Box<mistral::Model>),
    Qwen3Vl(Box<qwen3_vl::Qwen3VLModel>),
    Gemma4(Box<gemma4::Model>),
    /// Text only, so the vision tower is never built.
    Gemma4Text(Box<gemma4::text::TextModel>),
}

impl Backend {
    /// Runs one forward pass. `image` is only given on the first pass, because
    /// the image tokens live in the prompt.
    fn forward(
        &mut self,
        tokens: &[u32],
        offset: usize,
        image: Option<&PreparedImage>,
        device: &Device,
    ) -> anyhow::Result<Tensor> {
        let input = Tensor::new(tokens, device)?.unsqueeze(0)?;
        let logits = match self {
            Self::Mistral(model) => model.forward(&input, offset)?,
            Self::Qwen3Vl(model) => {
                let (pixels, grid, spans) = match image {
                    Some(PreparedImage::Qwen { patches, grid, .. }) => {
                        let spans = image_spans(tokens, QWEN_IMAGE_PAD);
                        (Some(patches.clone()), Some(grid.clone()), vec![spans])
                    }
                    _ => (None, None, vec![Vec::new()]),
                };
                model.forward(
                    &input,
                    pixels,
                    None,
                    grid,
                    None,
                    vec![tokens.len()],
                    spans,
                    vec![Vec::new()],
                    &[offset],
                )?
            }
            Self::Gemma4Text(model) => model.forward(&input, offset)?,
            Self::Gemma4(model) => match image {
                Some(PreparedImage::Gemma { pixels, .. }) => {
                    let batch = [pixels.clone()];
                    model.forward_multimodal(&input, Some(&batch), None, None, offset)?
                }
                _ => model.forward(&input, offset)?,
            },
        };
        // The models disagree on the rank they return: Mistral gives
        // (batch, 1, vocab), Qwen takes the last position itself and gives
        // (batch, vocab). Reduce to a flat vocab vector either way.
        let mut logits = logits;
        while logits.rank() > 1 {
            logits = logits.squeeze(0)?;
        }
        Ok(logits)
    }
}

const QWEN_IMAGE_PAD: u32 = 151_655;
const QWEN_VISION_START: u32 = 151_652;
const QWEN_VISION_END: u32 = 151_653;

/// Finds the contiguous runs of `needle` in `tokens`, as half-open ranges.
fn image_spans(tokens: &[u32], needle: u32) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut start = None;
    for (i, &t) in tokens.iter().enumerate() {
        match (t == needle, start) {
            (true, None) => start = Some(i),
            (false, Some(s)) => {
                spans.push((s, i));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(s) = start {
        spans.push((s, tokens.len()));
    }
    spans
}

fn read_proc_value(key: &str) -> u64 {
    fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status
                .lines()
                .find(|line| line.starts_with(key))
                .and_then(|line| {
                    line.split_whitespace()
                        .nth(1)
                        .and_then(|kb| kb.parse::<u64>().ok())
                })
        })
        .map_or(0, |kb| kb * 1024)
}

async fn fetch_item(args: &Args) -> anyhow::Result<Item> {
    fs::create_dir_all(&args.cache_dir)?;
    let path = args.cache_dir.join(format!("item_{}.json", args.item));
    if !path.exists() {
        let url = format!("{}/api/item/{}", args.api_base, args.item);
        let body = reqwest::get(&url).await?.error_for_status()?.bytes().await?;
        fs::write(&path, &body)?;
    }
    Ok(serde_json::from_slice(&fs::read(&path)?)?)
}

async fn fetch_preview(args: &Args, item: &Item) -> anyhow::Result<PathBuf> {
    let url = item
        .preview_url
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("item {} has no preview_url", args.item))?;
    let path = args.cache_dir.join(format!("preview_{}", args.item));
    if !path.exists() {
        let body = reqwest::get(url).await?.error_for_status()?.bytes().await?;
        fs::write(&path, &body)?;
    }
    Ok(path)
}

/// Resizes so that both sides are a multiple of `factor` and the pixel count
/// stays inside the bounds. This is the Qwen2-VL "smart resize".
fn smart_resize(h: usize, w: usize, factor: usize, min_px: usize, max_px: usize) -> (usize, usize) {
    let round_to = |v: usize| ((v as f64 / factor as f64).round() as usize).max(1) * factor;
    let (mut rh, mut rw) = (round_to(h), round_to(w));
    if rh * rw > max_px {
        let beta = ((h * w) as f64 / max_px as f64).sqrt();
        rh = ((h as f64 / beta / factor as f64).floor() as usize).max(1) * factor;
        rw = ((w as f64 / beta / factor as f64).floor() as usize).max(1) * factor;
    } else if rh * rw < min_px {
        let beta = (min_px as f64 / (h * w) as f64).sqrt();
        rh = ((h as f64 * beta / factor as f64).ceil() as usize) * factor;
        rw = ((w as f64 * beta / factor as f64).ceil() as usize) * factor;
    }
    (rh, rw)
}

/// Builds the Qwen3-VL patch tensor: normalise to mean 0.5 / std 0.5, then
/// flatten to (num_patches, channels * temporal * patch * patch).
fn prepare_qwen_image(path: &Path, dtype: DType, device: &Device) -> anyhow::Result<PreparedImage> {
    const PATCH: usize = 16;
    const MERGE: usize = 2;
    const TEMPORAL: usize = 2;

    // The cached preview has no extension, so sniff the format from content.
    let img = image::ImageReader::open(path)?
        .with_guessed_format()?
        .decode()?
        .to_rgb8();
    let (w0, h0) = (img.width() as usize, img.height() as usize);
    let (h, w) = smart_resize(h0, w0, PATCH * MERGE, 65_536, 16_777_216);
    let img = image::imageops::resize(
        &img,
        w as u32,
        h as u32,
        image::imageops::FilterType::CatmullRom,
    );

    let data: Vec<f32> = img
        .into_raw()
        .into_iter()
        .map(|b| (f32::from(b) / 255.0 - 0.5) / 0.5)
        .collect();
    // (h, w, c) -> (c, h, w)
    let pixels = Tensor::from_vec(data, (h, w, 3), device)?.permute((2, 0, 1))?;
    // The temporal patch size is 2, so a still image is repeated once.
    let pixels = Tensor::stack(&[&pixels, &pixels], 0)?; // (t, c, h, w)

    let (grid_h, grid_w) = (h / PATCH, w / PATCH);
    let patches = pixels
        .reshape((TEMPORAL, 3, grid_h, PATCH, grid_w, PATCH))?
        .permute((2, 4, 1, 0, 3, 5))? // (grid_h, grid_w, c, t, ph, pw)
        .reshape((grid_h * grid_w, 3 * TEMPORAL * PATCH * PATCH))?
        .to_dtype(dtype)?;

    let grid = Tensor::new(&[1u32, grid_h as u32, grid_w as u32], device)?.reshape((1, 3))?;
    Ok(PreparedImage::Qwen {
        patches,
        grid,
        tokens: (grid_h * grid_w) / (MERGE * MERGE),
    })
}

/// Builds the Gemma 4 pixel tensor. The tower pools patches by a factor of
/// `pooling_kernel_size` squared, so the patch count has to divide by 9.
fn prepare_gemma_image(path: &Path, dtype: DType, device: &Device) -> anyhow::Result<PreparedImage> {
    const PATCH: usize = 16;
    const POOL: usize = 3;
    const TARGET_PATCHES: f64 = 2520.0; // 280 soft tokens, as the processor uses

    // The cached preview has no extension, so sniff the format from content.
    let img = image::ImageReader::open(path)?
        .with_guessed_format()?
        .decode()?
        .to_rgb8();
    let (w0, h0) = (f64::from(img.width()), f64::from(img.height()));
    let scale = (TARGET_PATCHES * (PATCH * PATCH) as f64 / (w0 * h0)).sqrt();
    // Round the patch counts to a multiple of the pooling kernel so that the
    // pooled length is exact.
    let round_patches = |px: f64| {
        let patches = (px * scale / PATCH as f64 / POOL as f64).round().max(1.0) as usize * POOL;
        patches
    };
    let (grid_h, grid_w) = (round_patches(h0), round_patches(w0));
    let (h, w) = (grid_h * PATCH, grid_w * PATCH);

    let img = image::imageops::resize(
        &img,
        w as u32,
        h as u32,
        image::imageops::FilterType::CatmullRom,
    );
    // The processor rescales by 1/255 and does not normalise further.
    let data: Vec<f32> = img
        .into_raw()
        .into_iter()
        .map(|b| f32::from(b) / 255.0)
        .collect();
    let pixels = Tensor::from_vec(data, (h, w, 3), device)?
        .permute((2, 0, 1))?
        .unsqueeze(0)?
        .to_dtype(dtype)?;

    Ok(PreparedImage::Gemma {
        pixels,
        tokens: (grid_h * grid_w) / (POOL * POOL),
    })
}

/// Renders the model's own chat template around the prompt, then tokenises,
/// splicing the image tokens in where the marker sits.
fn build_tokens(
    tokenizer: &Tokenizer,
    template: &ChatTemplate,
    user_content: &str,
    model: ModelKey,
    image: Option<&PreparedImage>,
    image_token_id: u32,
) -> anyhow::Result<Vec<u32>> {
    use candle_examples::chat_template::Message;

    let rendered = template
        .apply_for_generation(&[Message::user(user_content)])
        .map_err(|e| anyhow::anyhow!("chat template: {e}"))?;

    let encode = |text: &str| -> anyhow::Result<Vec<u32>> {
        Ok(tokenizer
            .encode(text, false)
            .map_err(|e| anyhow::anyhow!("tokenise: {e}"))?
            .get_ids()
            .to_vec())
    };

    let Some(prepared) = image else {
        return encode(&rendered.replace(IMAGE_MARKER, ""));
    };

    let (head, tail) = rendered
        .split_once(IMAGE_MARKER)
        .ok_or_else(|| anyhow::anyhow!("the chat template dropped the image marker"))?;

    let mut tokens = encode(head)?;
    match model {
        ModelKey::Qwen3Vl => {
            tokens.push(QWEN_VISION_START);
            tokens.extend(std::iter::repeat_n(image_token_id, prepared.tokens()));
            tokens.push(QWEN_VISION_END);
        }
        _ => tokens.extend(std::iter::repeat_n(image_token_id, prepared.tokens())),
    }
    tokens.extend(encode(tail)?);
    Ok(tokens)
}

/// candle's `qwen3_vl` builds `lm_head` with `linear`, which demands a bias
/// tensor that no Qwen3 checkpoint carries (`text.rs:285`; it should be
/// `linear_no_bias`). A zero bias is arithmetically identical to no bias, so
/// write one into a small side file and let the VarBuilder find it there.
fn qwen_lm_head_shim(
    cache_dir: &Path,
    vocab_size: usize,
    dtype: DType,
) -> anyhow::Result<PathBuf> {
    let path = cache_dir.join(format!("qwen3vl_lm_head_bias_{vocab_size}_{dtype:?}.safetensors"));
    if !path.exists() {
        let zeros = Tensor::zeros(vocab_size, dtype, &Device::Cpu)?;
        let mut map = std::collections::HashMap::new();
        map.insert("lm_head.bias".to_string(), zeros);
        candle_core::safetensors::save(&map, &path)?;
    }
    Ok(path)
}

/// Collects the weight files, handling both the sharded and the single-file
/// layouts. Gemma 4 ships one `model.safetensors` with no index.
async fn weight_files(repo: &hf_hub::api::tokio::ApiRepo) -> anyhow::Result<Vec<PathBuf>> {
    match repo.get("model.safetensors.index.json").await {
        Ok(index) => {
            let json: serde_json::Value = serde_json::from_slice(&fs::read(index)?)?;
            let map = json
                .get("weight_map")
                .and_then(|m| m.as_object())
                .ok_or_else(|| anyhow::anyhow!("no weight_map in the index"))?;
            let mut names: Vec<&str> = map.values().filter_map(|v| v.as_str()).collect();
            names.sort_unstable();
            names.dedup();
            let mut files = Vec::with_capacity(names.len());
            for name in names {
                files.push(repo.get(name).await?);
            }
            Ok(files)
        }
        Err(_) => Ok(vec![repo.get("model.safetensors").await?]),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let started = Instant::now();

    if args.image && !args.model.supports_vision() {
        anyhow::bail!("{:?} has no vision tower", args.model);
    }

    let repo_id = args
        .repo
        .clone()
        .unwrap_or_else(|| args.model.default_repo().to_string());

    let item = fetch_item(&args).await?;
    let prompt_template = fs::read_to_string(args.prompt_dir.join(args.prompt.file_name()))?;
    let mut user_content = classification::populate_prompt(
        &prompt_template,
        &item.title,
        &item.description,
    );
    if args.image {
        user_content = format!("{IMAGE_MARKER}\n{user_content}");
    }

    let api = Api::new()?;
    let repo = api.repo(Repo::with_revision(
        repo_id.clone(),
        RepoType::Model,
        "main".to_string(),
    ));

    let files = weight_files(&repo).await?;
    let weight_bytes = files
        .iter()
        .filter_map(|f| fs::metadata(f).ok())
        .map(|m| m.len())
        .sum();

    if args.download_only {
        for extra in ["tokenizer.json", "tokenizer_config.json", "config.json"] {
            let _ = repo.get(extra).await;
        }
        let _ = repo.get("chat_template.jinja").await;
        println!(
            "{repo_id}: {} files, {:.1} GiB",
            files.len(),
            weight_bytes as f64 / 1024.0 / 1024.0 / 1024.0
        );
        return Ok(());
    }

    let tokenizer_start = Instant::now();
    let tokenizer = Tokenizer::from_file(repo.get("tokenizer.json").await?)
        .map_err(|e| anyhow::anyhow!("tokenizer: {e}"))?;
    let template = match ChatTemplate::from_tokenizer_config(repo.get("tokenizer_config.json").await?)
    {
        Ok(template) => template,
        // Gemma 4 keeps the template in its own file, not in tokenizer_config.
        Err(_) => {
            let jinja = fs::read_to_string(repo.get("chat_template.jinja").await?)?;
            ChatTemplate::new(jinja, "<bos>", "<eos>")
                .map_err(|e| anyhow::anyhow!("chat template: {e}"))?
        }
    };
    // Gemma 4's shipped template calls `.get()` on a message map, which
    // minijinja has no method for. For a single user turn the template reduces
    // to the three markers below, so render that instead of failing.
    let template = if template
        .apply_for_generation(&[candle_examples::chat_template::Message::user("probe")])
        .is_err()
    {
        const MINIMAL: &str = concat!(
            "{{- bos_token -}}",
            "{%- for m in messages -%}",
            "{{- '<|turn>' + m.role + '\n' + m.content + '<turn|>\n' -}}",
            "{%- endfor -%}",
            "{%- if add_generation_prompt -%}{{- '<|turn>model\n' -}}{%- endif -%}",
        );
        ChatTemplate::new(MINIMAL, "<bos>", "<eos>")
            .map_err(|e| anyhow::anyhow!("fallback chat template: {e}"))?
    } else {
        template
    };
    let tokenizer_secs = tokenizer_start.elapsed().as_secs_f64();

    let raw_config: serde_json::Value =
        serde_json::from_slice(&fs::read(repo.get("config.json").await?)?)?;

    let device = Device::Cpu;
    let dtype: DType = args.dtype.into();

    let load_start = Instant::now();
    let (mut backend, image_token_id) = match args.model {
        ModelKey::Mistral => {
            let config: mistral::Config = serde_json::from_value(raw_config)?;
            let vb = unsafe { VarBuilder::from_mmaped_safetensors(&files, dtype, &device)? };
            (Backend::Mistral(Box::new(mistral::Model::new(&config, vb)?)), 0)
        }
        ModelKey::Qwen3Vl => {
            // The published config keeps `tie_word_embeddings` at the top
            // level, but candle's TextConfig demands it inside text_config.
            let mut patched = raw_config.clone();
            let tie = patched
                .get("tie_word_embeddings")
                .cloned()
                .unwrap_or(serde_json::Value::Bool(false));
            if let Some(text) = patched.get_mut("text_config").and_then(|t| t.as_object_mut()) {
                text.entry("tie_word_embeddings").or_insert(tie);
                text.entry("sliding_window").or_insert(serde_json::Value::Null);
            }
            let config: qwen3_vl::Config = serde_json::from_value(patched)?;
            let image_token_id = config.image_token_id;
            let mut files = files.clone();
            if !config.text_config.tie_word_embeddings {
                files.push(qwen_lm_head_shim(
                    &args.cache_dir,
                    config.text_config.vocab_size,
                    dtype,
                )?);
            }
            let vb = unsafe { VarBuilder::from_mmaped_safetensors(&files, dtype, &device)? };
            (
                Backend::Qwen3Vl(Box::new(qwen3_vl::Qwen3VLModel::new(&config, vb)?)),
                image_token_id,
            )
        }
        ModelKey::Gemma4 => {
            let config: gemma4::config::Gemma4Config = serde_json::from_value(raw_config.clone())?;
            let image_token_id = u32::try_from(config.image_token_id)?;
            let vb = unsafe { VarBuilder::from_mmaped_safetensors(&files, dtype, &device)? };
            if args.image {
                // candle's gemma4 pushes "model" twice for the language model,
                // which does not match the published checkpoints.
                let vb = vb.rename_f(|name: &str| {
                    name.replace("model.language_model.model.", "model.language_model.")
                });
                (
                    Backend::Gemma4(Box::new(gemma4::Model::new(&config, vb)?)),
                    image_token_id,
                )
            } else {
                // The vision tower cannot load these weights at all: candle
                // asks for `q_proj.weight` while the checkpoint stores the
                // clipped form, `q_proj.linear.weight` plus its min/max pair.
                // Text-only runs skip the tower entirely.
                let vb = vb.rename_f(|name: &str| {
                    name.strip_prefix("model.")
                        .map_or_else(|| name.to_string(), |rest| format!("model.language_model.{rest}"))
                });
                (
                    Backend::Gemma4Text(Box::new(gemma4::text::TextModel::new(
                        &config.text_config,
                        vb,
                    )?)),
                    image_token_id,
                )
            }
        }
    };
    let load_secs = load_start.elapsed().as_secs_f64();

    let prepared = if args.image {
        let path = fetch_preview(&args, &item).await?;
        Some(match args.model {
            ModelKey::Qwen3Vl => prepare_qwen_image(&path, dtype, &device)?,
            _ => prepare_gemma_image(&path, dtype, &device)?,
        })
    } else {
        None
    };

    let tokens = build_tokens(
        &tokenizer,
        &template,
        &user_content,
        args.model,
        prepared.as_ref(),
        image_token_id,
    )?;

    let stop_ids: Vec<(u32, String)> = args
        .model
        .stop_tokens()
        .iter()
        .filter_map(|t| tokenizer.token_to_id(t).map(|id| (id, (*t).to_string())))
        .collect();
    if stop_ids.is_empty() {
        anyhow::bail!("none of {:?} are in the tokenizer", args.model.stop_tokens());
    }

    let mut stream = TokenOutputStream::new(tokenizer.clone());
    let mut logits_processor = LogitsProcessor::from_sampling(299_792_458, Sampling::ArgMax);

    let generate_start = Instant::now();
    let mut all_tokens = tokens.clone();
    let prefill_start = Instant::now();
    let mut logits = backend.forward(&tokens, 0, prepared.as_ref(), &device)?;
    let prefill_secs = prefill_start.elapsed().as_secs_f64();

    let mut output = String::new();
    let mut decode_tokens = 0usize;
    let mut stop_token = None;
    let decode_start = Instant::now();
    loop {
        let scored = logits.to_dtype(DType::F32)?;
        let scored = if args.repeat_penalty == 1.0 {
            scored
        } else {
            let start = all_tokens.len().saturating_sub(args.repeat_last_n);
            candle_transformers::utils::apply_repeat_penalty(
                &scored,
                args.repeat_penalty,
                &all_tokens[start..],
            )?
        };
        let next = logits_processor.sample(&scored)?;
        all_tokens.push(next);
        if let Some((_, name)) = stop_ids.iter().find(|(id, _)| *id == next) {
            stop_token = Some(name.clone());
            break;
        }
        if let Some(piece) = stream
            .next_token(next)
            .map_err(|e| anyhow::anyhow!("stream: {e}"))?
        {
            output.push_str(&piece);
        }
        decode_tokens += 1;
        if decode_tokens >= args.max_tokens {
            break;
        }
        logits = backend.forward(&[next], all_tokens.len() - 1, None, &device)?;
    }
    let decode_secs = decode_start.elapsed().as_secs_f64();
    if let Some(rest) = stream
        .decode_rest()
        .map_err(|e| anyhow::anyhow!("stream: {e}"))?
    {
        output.push_str(&rest);
    }
    let generate_secs = generate_start.elapsed().as_secs_f64();

    // No `sanitise_output` here: its replacements were tuned against Mistral 7B
    // and would flatter or mangle any other model. Judge the raw answer.
    let parses_as_json =
        serde_json::from_str::<classification::MLProperties>(output.trim()).is_ok();

    let report = RunReport {
        model: args.model,
        repo: repo_id,
        item: args.item.clone(),
        title: item.title.clone(),
        prompt: args.prompt,
        with_image: args.image,
        dtype: args.dtype,
        label: args.label.clone(),
        threads: std::env::var("OMP_NUM_THREADS").ok(),
        repeat_penalty: args.repeat_penalty,
        weight_bytes,
        load_secs,
        tokenizer_secs,
        prompt_chars: user_content.len(),
        prefill_tokens: tokens.len(),
        image_tokens: prepared.as_ref().map_or(0, PreparedImage::tokens),
        prefill_secs,
        decode_tokens,
        decode_secs,
        decode_tokens_per_sec: if decode_secs > 0.0 {
            decode_tokens as f64 / decode_secs
        } else {
            0.0
        },
        generate_secs,
        total_secs: started.elapsed().as_secs_f64(),
        hit_token_cap: decode_tokens >= args.max_tokens,
        stop_token,
        peak_rss_bytes: read_proc_value("VmHWM:"),
        final_rss_bytes: read_proc_value("VmRSS:"),
        raw_output: output,
        parses_as_json,
    };

    fs::create_dir_all(&args.out_dir)?;
    let modality = if args.image { "image" } else { "text" };
    let name = format!(
        "{:?}_{}_{:?}_{}.json",
        report.model, report.item, report.prompt, modality
    )
    .to_lowercase();
    fs::write(
        args.out_dir.join(&name),
        serde_json::to_string_pretty(&report)?,
    )?;

    println!(
        "{:?} {} {:?} {modality}: load {:.1}s, prefill {} tok in {:.1}s, decode {} tok at {:.2} tok/s, peak RSS {:.1} GiB, json={} -> {name}",
        report.model,
        report.item,
        report.prompt,
        report.load_secs,
        report.prefill_tokens,
        report.prefill_secs,
        report.decode_tokens,
        report.decode_tokens_per_sec,
        report.peak_rss_bytes as f64 / 1024.0 / 1024.0 / 1024.0,
        report.parses_as_json,
    );
    Ok(())
}
