use crate::backend::BackendKind;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Secret {
    pub name: String,
    pub hash: String,
    pub backend_hint: Option<BackendKind>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct SecretContent(pub Vec<u8>);

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ProvisionedSecret<'a> {
    pub secret: &'a Secret,
    pub content: SecretContent,
    pub path: PathBuf,
}

impl std::fmt::Debug for SecretContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", String::from_utf8_lossy(&self.0))
    }
}

impl AsRef<[u8]> for SecretContent {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<Secret> for Secret {
    fn as_ref(&self) -> &Secret {
        self
    }
}
