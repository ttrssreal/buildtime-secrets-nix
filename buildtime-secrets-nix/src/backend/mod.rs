pub mod executable;
pub mod sops;

use crate::error::Result;
use crate::secret::{Secret, SecretContent};
use crate::{Config, Error};
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackendKind {
    Sops,
    Executable,
}

impl std::fmt::Display for BackendKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{self:#?}")
    }
}

pub trait Backend<'a> {
    fn provision(&self, secret: &Secret) -> Option<SecretContent>;
}

/// Instantiate a new `backend_kind` backend.
///
/// # Errors
///
/// If the corrosponding constructor fails.
pub fn create<'a>(
    backend_kind: BackendKind,
    config: &'a Config,
) -> Result<Box<dyn Backend<'a> + 'a>> {
    debug!("creating backend {backend_kind}");
    match backend_kind {
        BackendKind::Sops => Ok(Box::new(sops::Sops::new(config)?)),
        BackendKind::Executable => Ok(Box::new(executable::Executable::new(config)?)),
    }
}

/// Ask a backend to validate `config` meets its
/// requirements.
pub fn validate_config(backend_kind: BackendKind, config: &Config) -> bool {
    debug!("validating config for {backend_kind}");
    match backend_kind {
        BackendKind::Sops => sops::Sops::validate_config(config),
        BackendKind::Executable => executable::Executable::validate_config(config),
    }
}

/// Parse out a specific backends configuration from
/// the global configuration.
///
/// # Errors
///
///  - If the `backend_config` field does not appear on the
///    global config.
///  - If the `backend_config.<backend_name>` field does not appear on the
///    global config.
///  - If the `backend_config.<backend_name>` field does not match the
///    expected structure.
pub fn get_backend_config<T: serde::de::DeserializeOwned>(
    config: &Config,
    backend_name: &str,
) -> Result<T> {
    let Some(backend_configs) = config.backend_config.clone() else {
        debug!("cant find \"backend_config\"");
        return Err(Error::NoConfigForBackends);
    };

    let Some(backend_config) = backend_configs.get(backend_name) else {
        debug!("cant find \"backend_config.{}\"", backend_name);
        return Err(Error::NoBackendConfig(backend_name.to_string()));
    };

    let parsed = match serde_json::from_value::<T>(backend_config.clone()) {
        Ok(parsed) => parsed,
        Err(err) => {
            debug!("failed to parse {backend_name} config: {err}");
            return Err(Error::NoBackendConfig(backend_name.to_string()));
        }
    };

    Ok(parsed)
}

pub fn provision_with_cmd(
    secret: &Secret,
    cmd: &mut std::process::Command,
) -> Option<SecretContent> {
    let decrypt_output = match cmd.output() {
        Ok(out) => out,
        Err(err) => {
            debug!("failed to instantiate executable: {err}");
            return None;
        }
    };

    if !decrypt_output.status.success() {
        let from_utf8_lossy = String::from_utf8_lossy;
        debug!("failed to decrypt secret with executable:");
        debug!("    stdout: {}", from_utf8_lossy(&decrypt_output.stdout));
        debug!("    stderr: {}", from_utf8_lossy(&decrypt_output.stderr));
        return None;
    }

    debug!("successfully decrypted secret {}", secret.name);

    Some(crate::secret::SecretContent(decrypt_output.stdout))
}
