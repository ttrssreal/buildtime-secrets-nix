use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub derivation: String,
    pub secret_dir: PathBuf,
    pub backend_config: Option<HashMap<String, serde_json::Value>>,
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{self:#?}")
    }
}
