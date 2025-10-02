use crate::backend::Backend;
use crate::secret::Secret;
use crate::secret::SecretContent;
use crate::{Config, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Serialize, Deserialize)]
pub struct BackendConfig {
    file: PathBuf,
}

/// A simple backend that accepts an arbitrary executable that,
/// when invoked, provisions a secret.
///
/// The name of the secret will be passed as the executables
/// first command line argument (argv[1]).
/// The executable will then write the full contents of the
/// secret to stdout.
pub struct Executable {
    config: BackendConfig,
}

impl Backend<'_> for Executable {
    fn provision(&self, secret: &Secret) -> Option<SecretContent> {
        let mut cmd = std::process::Command::new(&self.config.file);
        cmd.arg(secret.name.clone());
        crate::backend::provision_with_cmd(secret, &mut cmd)
    }
}

impl Executable {
    /// Creates a new Executable backend.
    ///
    /// # Errors
    ///
    /// If the associated config can't be parsed.
    pub fn new(root_config: &Config) -> Result<Self> {
        let config =
            crate::backend::get_backend_config::<BackendConfig>(root_config, "executable")?;

        Ok(Executable { config })
    }

    /// Validate an exacutable backend config.
    #[must_use]
    pub fn validate_config(root_config: &Config) -> bool {
        let parse_result =
            crate::backend::get_backend_config::<BackendConfig>(root_config, "executable");

        let Ok(_config) = parse_result else {
            return false;
        };

        true
    }
}
