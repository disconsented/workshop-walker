use std::collections::HashSet;

use hf_hub::api::tokio::ApiRepo;
use snafu::{ResultExt, Whatever, whatever};
use tokio::{fs::File, io::AsyncReadExt};
use tracing::debug;

/// Loads the safetensors files for a model from the hub based on a json index
/// file.
pub async fn hub_load_safetensors(
    repo: &ApiRepo,
    json_file: &str,
) -> Result<Vec<std::path::PathBuf>, Whatever> {
    debug!(file = json_file, "downloading safetensors");
    let json_file = repo
        .get(json_file)
        .await
        .whatever_context("get json_file")?;
    let mut json_file = File::open(json_file).await.whatever_context("open")?;
    let mut bytes = vec![0u8; 0];
    json_file
        .read_to_end(&mut bytes)
        .await
        .whatever_context("reading file")?;
    let json: serde_json::Value =
        serde_json::from_slice(bytes.as_slice()).whatever_context("deserializing")?;
    let weight_map = match json.get("weight_map") {
        None => whatever!("no weight map in {json_file:?}"),
        Some(serde_json::Value::Object(map)) => map,
        Some(_) => whatever!("weight map in {json_file:?} is not a map"),
    };
    let mut safetensors_files = HashSet::new();
    for value in weight_map.values() {
        if let Some(file) = value.as_str() {
            safetensors_files.insert(repo.get(file).await.unwrap());
        }
    }

    Ok(safetensors_files.into_iter().collect())
}
