use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort, async_trait};
use reqwest::{Client, Response, StatusCode, header::CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use snafu::{OptionExt, ResultExt, Snafu};
use tokio::{fs::read_to_string, time::sleep};
use tokio_stream::{self as stream, StreamExt};
use tracing::{Instrument, debug, debug_span, instrument, warn};

/// The pause between two health checks while the server loads its model.
const HEALTH_POLL: Duration = Duration::from_millis(250);
/// The time the actor waits for the server before it gives up.
const HEALTH_BUDGET: Duration = Duration::from_secs(300);

/// One request to the model. `name` goes into the JSON schema, `fields` names
/// the arrays the answer must hold.
#[derive(Clone, Copy, Debug)]
struct Task {
    name: &'static str,
    prompt_path: &'static str,
    fields: [&'static str; 2],
}

const TASKS: [Task; 2] = [
    Task {
        name: "features",
        prompt_path: "./prompts/features.txt",
        fields: ["types", "features"],
    },
    Task {
        name: "genres",
        prompt_path: "./prompts/genres.txt",
        fields: ["genres", "themes"],
    },
];

pub struct LlamaActor;

pub struct LlamaArgs {
    pub client: Client,
    pub api_url: String,
}

pub struct LlamaState {
    client: Client,
    api_url: String,
    /// Each task with the prompt template read at start up.
    prompts: Vec<(Task, String)>,
}

/// What the actor takes.
pub type LlamaMsg = LlamaRequest;

pub enum LlamaRequest {
    Process {
        title: String,
        description: String,
        rpc_reply_port: RpcReplyPort<Result<MLProperties, LlamaError>>,
    },
}

#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum LlamaError {
    #[snafu(display("The {task} request to the model server failed"))]
    Request {
        task: &'static str,
        source: reqwest::Error,
    },
    #[snafu(display("The {task} response is not a chat completion"))]
    Decode {
        task: &'static str,
        source: reqwest::Error,
    },
    #[snafu(display("The model gave no {task} answer, because it stopped on {finish_reason:?}"))]
    NoContent {
        task: &'static str,
        finish_reason: Option<FinishReason>,
    },
    #[snafu(display("The {task} answer does not agree with its schema"))]
    Parse {
        task: &'static str,
        source: serde_json::Error,
    },
}

#[derive(Debug, Snafu)]
#[non_exhaustive]
enum StartupError {
    #[snafu(display("The model server at {api_url} stayed unavailable for {} seconds", HEALTH_BUDGET.as_secs()))]
    Unavailable { api_url: String },
    #[snafu(display("The health check of the model server at {api_url} gave {status}"))]
    Unhealthy { api_url: String, status: StatusCode },
    #[snafu(display("Reading the prompt at {path}"))]
    Prompt {
        path: &'static str,
        source: std::io::Error,
    },
}

#[async_trait]
impl Actor for LlamaActor {
    type Arguments = LlamaArgs;
    type Msg = LlamaMsg;
    type State = LlamaState;

    #[tracing::instrument(level = "debug", skip_all)]
    async fn pre_start(
        &self,
        _: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        wait_until_healthy(&args.client, &args.api_url).await?;

        let prompts = stream::iter(TASKS)
            .then(async |task| {
                let template = read_to_string(task.prompt_path)
                    .await
                    .context(PromptSnafu {
                        path: task.prompt_path,
                    })?;
                Ok((task, template))
            })
            .collect::<Result<Vec<_>, StartupError>>()
            .await?;

        Ok(LlamaState {
            client: args.client,
            api_url: args.api_url,
            prompts,
        })
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        let LlamaRequest::Process {
            title,
            description,
            rpc_reply_port,
        } = message;

        let output = run_process(state, &title, &description)
            .instrument(debug_span!("llama process", %title))
            .await;
        debug!(%title, ?output, "got back props");
        let _ = rpc_reply_port.send(output);

        Ok(())
    }
}

/// Polls `/health` until the server answers, because llama.cpp refuses work
/// while it loads a model, and it does not listen at all before that.
#[instrument(skip(client))]
async fn wait_until_healthy(client: &Client, api_url: &str) -> Result<(), StartupError> {
    let started = Instant::now();
    loop {
        match client.get(format!("{api_url}/health")).send().await {
            Ok(response) if response.status().is_success() => {
                debug!("llama.cpp server is reporting healthy");
                return Ok(());
            }
            Ok(response) if response.status() == StatusCode::SERVICE_UNAVAILABLE => {
                debug!("llama.cpp server is still loading");
            }
            Ok(response) => {
                return UnhealthySnafu {
                    api_url,
                    status: response.status(),
                }
                .fail();
            }
            Err(error) => warn!(?error, "the health check did not reach the server"),
        }

        if started.elapsed() >= HEALTH_BUDGET {
            return UnavailableSnafu { api_url }.fail();
        }
        sleep(HEALTH_POLL).await;
    }
}

#[tracing::instrument(level = "debug", skip(state, title, description))]
/// Runs each task against the model and collects the answers into one set of
/// properties.
async fn run_process(
    state: &LlamaState,
    title: &str,
    description: &str,
) -> Result<MLProperties, LlamaError> {
    let mut properties = MLProperties::default();

    for (task, template) in &state.prompts {
        let prompt = populate_prompt(template, title, description);
        properties.merge(extract(state, *task, &prompt).await?);
    }

    Ok(properties)
}

#[instrument(skip_all, fields(task = task.name))]
async fn extract(state: &LlamaState, task: Task, prompt: &str) -> Result<MLProperties, LlamaError> {
    // reqwest carries no spans of its own, so without these the call is a gap
    // in the trace.
    let completion: ChatCompletion = state
        .client
        .post(format!("{}/v1/chat/completions", state.api_url))
        .header(CONTENT_TYPE, "application/json")
        .json(&request_body(task, prompt))
        .send()
        .instrument(debug_span!("chat completion"))
        .await
        .and_then(Response::error_for_status)
        .context(RequestSnafu { task: task.name })?
        .json()
        .instrument(debug_span!("decode completion"))
        .await
        .context(DecodeSnafu { task: task.name })?;

    let [
        Choice {
            message,
            finish_reason,
            ..
        },
    ] = completion.choices;
    debug!(?finish_reason, content = ?message.content, "model answered");

    let answer = message.content.context(NoContentSnafu {
        task: task.name,
        finish_reason,
    })?;
    serde_json::from_str(&answer).context(ParseSnafu { task: task.name })
}

/// Builds the chat completion request. The JSON schema holds the model to two
/// arrays of strings, one per field of the task.
fn request_body(task: Task, prompt: &str) -> ChatRequest<'_> {
    ChatRequest {
        messages: [RequestMessage {
            role: "user",
            content: prompt,
        }],
        response_format: ResponseFormat {
            json_schema: JsonSchema {
                name: task.name,
                schema: ObjectSchema {
                    properties: task
                        .fields
                        .iter()
                        .map(|field| (*field, ArraySchema::default()))
                        .collect(),
                    required: task.fields,
                    ..ObjectSchema::default()
                },
                ..JsonSchema::default()
            },
            ..ResponseFormat::default()
        },
        ..ChatRequest::default()
    }
}

/// One chat completion request. The default holds the sampling settings that
/// every task shares.
#[derive(Serialize, Debug)]
struct ChatRequest<'a> {
    messages: [RequestMessage<'a>; 1],
    temperature: f32,
    max_tokens: u32,
    response_format: ResponseFormat,
}

impl Default for ChatRequest<'_> {
    fn default() -> Self {
        Self {
            messages: [RequestMessage::default()],
            temperature: 0.0,
            max_tokens: 512,
            response_format: ResponseFormat::default(),
        }
    }
}

#[derive(Serialize, Debug, Default)]
struct RequestMessage<'a> {
    role: &'a str,
    content: &'a str,
}

/// The `OpenAI` structured output wrapper.
#[derive(Serialize, Debug)]
struct ResponseFormat {
    #[serde(rename = "type")]
    kind: &'static str,
    json_schema: JsonSchema,
}

impl Default for ResponseFormat {
    fn default() -> Self {
        Self {
            kind: "json_schema",
            json_schema: JsonSchema::default(),
        }
    }
}

#[derive(Serialize, Debug)]
struct JsonSchema {
    name: &'static str,
    strict: bool,
    schema: ObjectSchema,
}

impl Default for JsonSchema {
    fn default() -> Self {
        Self {
            name: "",
            strict: true,
            schema: ObjectSchema::default(),
        }
    }
}

/// The object the answer must match: one array of strings per field.
#[derive(Serialize, Debug)]
struct ObjectSchema {
    #[serde(rename = "type")]
    kind: &'static str,
    properties: BTreeMap<&'static str, ArraySchema>,
    required: [&'static str; 2],
    #[serde(rename = "additionalProperties")]
    additional_properties: bool,
}

impl Default for ObjectSchema {
    fn default() -> Self {
        Self {
            kind: "object",
            properties: BTreeMap::new(),
            required: [""; 2],
            additional_properties: false,
        }
    }
}

#[derive(Serialize, Debug)]
struct ArraySchema {
    #[serde(rename = "type")]
    kind: &'static str,
    items: ItemSchema,
}

impl Default for ArraySchema {
    fn default() -> Self {
        Self {
            kind: "array",
            items: ItemSchema::default(),
        }
    }
}

#[derive(Serialize, Debug)]
struct ItemSchema {
    #[serde(rename = "type")]
    kind: &'static str,
}

impl Default for ItemSchema {
    fn default() -> Self {
        Self { kind: "string" }
    }
}

pub fn populate_prompt(prompt: &str, title: &str, description: &str) -> String {
    prompt
        .replace("[TITLE]", title)
        .replace("[DESCRIPTION]", description)
        .replace('\t', "")
}

#[derive(Serialize, Deserialize, Debug, Default)]
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

impl MLProperties {
    /// Adds the other answer to this one. Each task fills two of the four
    /// fields and leaves the rest empty, so the tasks never overwrite each
    /// other.
    fn merge(&mut self, other: Self) {
        self.genres.extend(other.genres);
        self.themes.extend(other.themes);
        self.types.extend(other.types);
        self.features.extend(other.features);
    }
}

#[derive(Deserialize, Debug)]
pub struct ChatCompletion {
    // Technically a vec but its broken if we get more than one response really
    pub choices: [Choice; 1],
    #[serde(default)]
    pub model: String,
    pub usage: Option<Usage>,
    /// A llama.cpp extension, absent from the `OpenAI` response.
    pub timings: Option<Timings>,
}

#[derive(Deserialize, Debug)]
pub struct Choice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: Option<FinishReason>,
}

#[derive(Deserialize, Debug)]
pub struct ChatMessage {
    pub role: String,
    /// Null when the generation stopped before it produced an answer.
    pub content: Option<String>,
    /// Only present when the server runs with reasoning enabled.
    pub reasoning_content: Option<String>,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    #[serde(other)]
    Other,
}

#[derive(Deserialize, Debug)]
pub struct Usage {
    #[serde(rename = "prompt_tokens")]
    pub prompt: u32,
    #[serde(rename = "completion_tokens")]
    pub completion: u32,
    #[serde(rename = "total_tokens")]
    pub total: u32,
}

#[derive(Deserialize, Debug)]
pub struct Timings {
    pub prompt_n: u32,
    pub prompt_ms: f64,
    /// Null when a cached prompt makes `prompt_n` zero.
    pub prompt_per_second: Option<f64>,
    pub predicted_n: u32,
    pub predicted_ms: f64,
    pub predicted_per_second: Option<f64>,
}
