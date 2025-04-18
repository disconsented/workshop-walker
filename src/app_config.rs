use std::sync::Arc;
use serde::{Deserialize, Serialize};
use veil::Redact;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub steam: Steam,
    pub database: Database,
    pub read_from_cache: bool,
    pub download_workshop: bool,
}
#[derive(Serialize, Deserialize, Redact)]
pub struct Steam {
    #[redact]
    pub api_token: Arc<String>,
    pub appid: u32,
}
#[derive(Serialize, Deserialize, Redact)]
pub struct Database {
    pub user: String,
    #[redact]
    pub password: String,
}
