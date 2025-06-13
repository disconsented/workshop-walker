use std::{collections::HashMap, str::FromStr, sync::Arc};

use biscuit_auth::PrivateKey;
use serde::{Deserialize, Deserializer};
use veil::Redact;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub steam: Steam,
    pub database: Database,
    pub read_from_cache: bool,
    pub download_workshop: bool,
    pub updater: bool,
    pub base_url: Arc<String>,
    pub biscuit: Arc<Biscuit>,
}
#[derive(Deserialize, Redact)]
pub struct Steam {
    #[redact]
    pub api_token: Arc<String>,
    pub appid: u32,
}
#[derive(Deserialize, Redact)]
pub struct Database {
    pub user: String,
    #[redact]
    pub password: String,
}

#[derive(Redact)]
#[redact(all)]
pub struct Biscuit {
    pub private_key: PrivateKey,
    // pub lifetime: Duration,
}

impl<'de> serde::Deserialize<'de> for Biscuit {
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
