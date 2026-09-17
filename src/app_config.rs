use std::{collections::HashMap, str::FromStr, sync::Arc};

use biscuit_auth::PrivateKey;
use serde::{Deserialize, Deserializer};
use veil::Redact;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub steam: Steam,
    pub database: Database,
    pub updater: bool,
    pub ml_extraction: ML,
    pub force_update: bool,
    pub base_url: Arc<String>,
    pub biscuit: Arc<BiscuitConfig>,
    pub admin_users: Vec<i64>,
}

#[derive(Deserialize, Debug)]
pub struct ML {
    pub enabled: bool,
    pub url: Arc<String>,
}
#[derive(Deserialize, Redact)]
pub struct Steam {
    #[redact]
    pub api_token: Arc<String>,
}
#[derive(Deserialize, Redact)]
pub struct Database {
    pub user: String,
    #[redact]
    pub password: String,
}

#[derive(Redact)]
#[redact(all)]
pub struct BiscuitConfig {
    pub private_key: PrivateKey,
}

impl<'de> serde::Deserialize<'de> for BiscuitConfig {
    #[tracing::instrument(level = "trace", skip(d))]
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let mut map: HashMap<String, String> = HashMap::deserialize(d)?;
        Ok(Self {
            private_key: map
                .remove("private_key")
                .as_deref()
                .map(FromStr::from_str)
                .unwrap()
                .unwrap(),
        })
    }
}

#[cfg(test)]
mod test {
    use biscuit_auth::KeyPair;

    #[test]
    fn test_keygen() {
        let pair = KeyPair::new();
        println!("{}", pair.public().print());
        println!("{}", pair.private().to_prefixed_string());
    }
}
