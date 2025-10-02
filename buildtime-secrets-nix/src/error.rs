use crate::Secret;
use std::io;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NixError(libnixstore::error::Error),
    ParseSecret(serde_json::Error),
    NoConfigForBackends,
    NoBackendConfig(String),
    NoSuccessfulBackends(Secret),
    CreateSecretFile { path: PathBuf, source: io::Error },
    WriteSecret { path: PathBuf, source: io::Error },
    StorePathIsNotDerivation,
    CreateDrvSecretDir { path: PathBuf, source: io::Error },
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::NixError(source) => Some(source),
            Error::ParseSecret(source) => Some(source),
            Error::CreateSecretFile { source, .. }
            | Error::WriteSecret { source, .. }
            | Error::CreateDrvSecretDir { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                Error::NixError(source) =>
                    format!("an error occurred while interfacing with nix: {source}"),
                Error::ParseSecret(source) => format!("failed to parse secret: {source}"),
                Error::NoConfigForBackends => "no \"backend_config\" in config".to_string(),
                Error::NoSuccessfulBackends(secret) =>
                    format!("no backends could decrypt the secret \"{}\"", secret.name),
                Error::NoBackendConfig(backend) => format!("no backend config for {backend}"),
                Error::CreateSecretFile { path, source } => format!(
                    "can't create secret file \"{}\": {source}",
                    path.to_string_lossy()
                ),
                Error::WriteSecret { path, source } => format!(
                    "can't write secret file \"{}\": {source}",
                    path.to_string_lossy()
                ),
                Error::StorePathIsNotDerivation => "store path is not a derivation".to_string(),
                Error::CreateDrvSecretDir { path, source } => format!(
                    "can't create derivation secret directory \"{}\": {source}",
                    path.to_string_lossy()
                ),
            }
        )
    }
}

impl From<libnixstore::error::Error> for Error {
    fn from(value: libnixstore::error::Error) -> Self {
        Error::NixError(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::ParseSecret(value)
    }
}
