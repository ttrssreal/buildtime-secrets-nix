use crate::backend::Backend;
use crate::secret::Secret;
use crate::secret::SecretContent;
use crate::{Config, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct BackendConfig {
    sops_file: PathBuf,
    environment: Option<HashMap<String, String>>,
}

/// This backend will ask
/// [sops](https://github.com/getsops/sops) to decrypt the
/// required secret.
///
/// A sops file that containing the encrypted secrets is
/// passed in the backend configuration, along with environment
/// variables that will be set in the sops process.
pub struct Sops {
    config: BackendConfig,
}

impl Backend<'_> for Sops {
    fn provision(&self, secret: &Secret) -> Option<SecretContent> {
        let mut cmd = std::process::Command::new("sops");
        cmd.args(["--extract", format!("[\"{}\"]", secret.name).as_ref()]);
        cmd.args(["-d".as_ref(), self.config.sops_file.as_os_str()]);
        cmd.env_clear();

        // Retain parent process' PATH
        cmd.envs(
            std::env::vars()
                .filter(|(key, _)| key == "PATH")
                .collect::<HashMap<_, _>>(),
        );

        if let Some(envs) = &self.config.environment {
            cmd.envs(envs);
        }

        crate::backend::provision_with_cmd(secret, &mut cmd)
    }
}

impl Sops {
    /// Creates a new Sops backend.
    ///
    /// # Errors
    ///
    /// If the associated config can't be parsed.
    pub fn new(root_config: &Config) -> Result<Self> {
        let config = crate::backend::get_backend_config(root_config, "sops")?;
        Ok(Sops { config })
    }

    /// Validate an exacutable backend config.
    #[must_use]
    pub fn validate_config(root_config: &Config) -> bool {
        let parse_result = crate::backend::get_backend_config::<BackendConfig>(root_config, "sops");
        let Ok(_config) = parse_result else {
            return false;
        };

        true
    }
}
