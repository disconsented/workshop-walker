use std::{sync::Arc, time::Duration};

use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort, async_trait};
use reqwest::{Client, StatusCode, header::CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use snafu::{OptionExt, ResultExt, Snafu, Whatever, whatever};
use tokio::{fs::read_to_string, time::sleep};
use tracing::{debug, error, info_span, instrument};

use crate::db::model::InternalWorkshopItem;

pub struct LlamaActor;

pub struct LlamaArgs {
    pub client: Client,
    pub api_url: String,
}
pub struct LlamaState {
    client: Client,
    api_url: String,
    features_prompt: String,
    genres_prompt: String,
}

pub enum LlamaMsg {
    Process {
        title: String,
        description: String,
        rpc_reply_port: RpcReplyPort<Result<MLProperties, Error>>,
    },
}

type Error = Whatever;

// #[derive(Debug, Snafu)]
// pub enum Error {
//
// }
#[async_trait]
impl Actor for LlamaActor {
    type Arguments = LlamaArgs;
    type Msg = LlamaMsg;
    type State = LlamaState;

    async fn pre_start(
        &self,
        _: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        loop {
            let response = args
                .client
                .get(format!("{}/health", args.api_url.as_str()))
                .send()
                .await?;
            if response.status().is_success() {
                debug!("llama.cpp server is reporting healthy");
                break;
            }

            if response.status() == StatusCode::SERVICE_UNAVAILABLE {
                sleep(Duration::from_millis(250)).await;
                continue;
            }

            // return Err(whatever!(""));
            error!(status_code = ?response.status(), ?response);
            todo!()
        }

        let features_prompt = read_to_string("./prompts/features.txt")
            .await
            .inspect_err(|error| error!(?error, "loading features prompt"))?;
        let genres_prompt = read_to_string("./prompts/genres.txt")
            .await
            .inspect_err(|error| error!(?error, "loading genres prompt"))?;
        Ok(Self::State {
            client: args.client,
            api_url: args.api_url,
            features_prompt,
            genres_prompt,
        })
    }

    #[instrument(skip_all)]
    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            LlamaMsg::Process {
                title,
                description,
                rpc_reply_port,
            } => {
                let output = run_process(state, &title, &description).await;
                debug!(?title, ?output, "got back props");
                let _ = rpc_reply_port.send(output);
            }
        }

        Ok(())
    }
}

async fn run_process(
    state: &LlamaState,
    title: &str,
    description: &str,
) -> Result<MLProperties, Error> {
    let span = info_span!("llama process");
    let _g = span.enter();
    let mut properties = MLProperties {
        genres: vec![],
        themes: vec![],
        types: vec![],
        features: vec![],
    };

    {
        let features_prompt = populate_prompt(&state.features_prompt, &title, &description);
        let prompt = Root {
            messages: vec![Struct {
                role: "user".to_string(),
                content: features_prompt,
            }],
            temperature: 0.0,
            max_tokens: 512,
            response_format: ResponseFormat {
                r#type: "json_schema".to_string(),
                json_schema: JsonSchema {
                    name: "features".to_string(),
                    strict: true,
                    schema: Schema {
                        r#type: "object".to_string(),
                        properties: FeaturesProperties {
                            types: Struct1 {
                                r#type: "array".to_string(),
                                items: Items {
                                    r#type: "string".to_string(),
                                },
                            },
                            features: Struct1 {
                                r#type: "array".to_string(),
                                items: Items {
                                    r#type: "string".to_string(),
                                },
                            },
                        },
                        required: vec!["types".into(), "features".into()],
                        additional_properties: false,
                    },
                },
            },
        };

        let response = state
            .client
            .post(format!("{}/v1/chat/completions", state.api_url.as_str()))
            .header(CONTENT_TYPE, "application/json")
            .json(&prompt)
            .send()
            .await
            .inspect_err(|error| error!(?error, "sending features prompt"))
            .whatever_context("")?;

        let completion: ChatCompletion = response
            .json()
            .await
            .inspect_err(|error| error!(?error, "getting features response"))
            .whatever_context("")?;

        let txt = completion.choices[0].message.content.clone().whatever_context("")?;
        debug!(%title, %txt, finish_reason = ?completion.choices[0].finish_reason, "features response");
        let raw_props: MLProperties = serde_json::from_str(&txt).whatever_context("")?;
        properties.features.extend(raw_props.features);
        properties.types.extend(raw_props.types);
    };

    {
        let genres_prompt = populate_prompt(&state.genres_prompt, &title, &description);
        let prompt = Root {
            messages: vec![Struct {
                role: "user".to_string(),
                content: genres_prompt,
            }],
            temperature: 0.0,
            max_tokens: 512,
            response_format: ResponseFormat {
                r#type: "json_schema".to_string(),
                json_schema: JsonSchema {
                    name: "genres".to_string(),
                    strict: true,
                    schema: Schema {
                        r#type: "object".to_string(),
                        properties: GenresProperties {
                            genres: Struct1 {
                                r#type: "array".to_string(),
                                items: Items {
                                    r#type: "string".to_string(),
                                },
                            },
                            themes: Struct1 {
                                r#type: "array".to_string(),
                                items: Items {
                                    r#type: "string".to_string(),
                                },
                            },
                        },
                        required: vec!["genres".into(), "themes".into()],
                        additional_properties: false,
                    },
                },
            },
        };

        let response = state
            .client
            .post(format!("{}/v1/chat/completions", state.api_url.as_str()))
            .header(CONTENT_TYPE, "application/json")
            .json(&prompt)
            .send()
            .await
            .inspect_err(|error| error!(?error, "sending genres prompt"))
            .whatever_context("")?;

        let completion: ChatCompletion = response
            .json()
            .await
            .inspect_err(|error| error!(?error, "getting features response"))
            .whatever_context("")?;

        let txt = completion.choices[0].message.content.clone().whatever_context("")?;
        debug!(%title, %txt, finish_reason = ?completion.choices[0].finish_reason, "genres response");
        let raw_props: MLProperties = serde_json::from_str(&txt).whatever_context("")?;
        properties.genres.extend(raw_props.genres);
        properties.themes.extend(raw_props.themes);
    };

    Ok(properties)
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

#[derive(Deserialize, Debug)]
pub struct ChatCompletion {
    // Technically a vec but its broken if we get more than one response really
    pub choices: [Choice; 1],
    #[serde(default)]
    pub model: String,
    pub usage: Option<Usage>,
    /// A llama.cpp extension, absent from the OpenAI response.
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
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
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

#[derive(Serialize, Deserialize)]
struct Items {
    #[serde(rename = "type")]
    pub r#type: String,
}

#[derive(Serialize, Deserialize)]
struct Struct1 {
    #[serde(rename = "type")]
    pub r#type: String,
    pub items: Items,
}

#[derive(Serialize, Deserialize)]
struct FeaturesProperties {
    pub types: Struct1,
    pub features: Struct1,
}

#[derive(Serialize, Deserialize)]
struct GenresProperties {
    pub genres: Struct1,
    pub themes: Struct1,
}

#[derive(Serialize, Deserialize)]
struct Schema<T> {
    #[serde(rename = "type")]
    pub r#type: String,
    pub properties: T,
    pub required: Vec<String>,
    #[serde(rename = "additionalProperties")]
    pub additional_properties: bool,
}

#[derive(Serialize, Deserialize)]
struct JsonSchema<T> {
    pub name: String,
    pub strict: bool,
    pub schema: Schema<T>,
}

#[derive(Serialize, Deserialize)]
struct ResponseFormat<T> {
    #[serde(rename = "type")]
    pub r#type: String,
    pub json_schema: JsonSchema<T>,
}

#[derive(Serialize, Deserialize)]
struct Struct {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Deserialize)]
struct Root<T> {
    pub messages: Vec<Struct>,
    pub temperature: f64,
    pub max_tokens: i64,
    pub response_format: ResponseFormat<T>,
}

pub fn populate_prompt(prompt: &str, title: &str, description: &str) -> String {
    prompt
        .replace("[TITLE]", title)
        .replace("[DESCRIPTION]", description)
        .replace("\t", "")
}
