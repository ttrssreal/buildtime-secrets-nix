#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod backend;
pub mod config;
pub mod error;
pub mod secret;

pub use config::Config;
pub use error::Error;
pub use secret::Secret;

use backend::BackendKind;
use error::Result;
use libnixstore::Store;
use secret::{ProvisionedSecret, SecretContent};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tracing::{debug, warn};

const BACKEND_KINDS: [BackendKind; 2] = [BackendKind::Sops, BackendKind::Executable];

/// The context used when provisioning a derivations
/// declared secrets.
pub struct Provisioner<'a> {
    config: &'a Config,
    store: libnixstore::Store,
    derivation: libnixstore::StorePath,
}

impl<'a> Provisioner<'a> {
    /// Create a new context for provisioning secrets
    /// for a derivation.
    ///
    /// # Errors
    ///
    /// If we can't parse the derivation path or get
    /// the derivation name.
    pub fn new(config: &'a Config) -> Result<Self> {
        let store = Store::new()?;

        let derivation = store.parse_store_path(&config.derivation)?;

        // https://github.com/tokio-rs/tracing/issues/2448
        // https://github.com/tokio-rs/tracing/issues/2704
        let derivation_name = store.derivation_name(&derivation)?;
        debug!("derivation name: {}", derivation_name);

        Ok(Self {
            config,
            store,
            derivation,
        })
    }

    /// Calculate the secret directory used for this
    /// derivation.
    ///
    /// # Errors
    ///
    /// If we can't get the path name of the derivation.
    pub fn derivation_secret_directory(&self) -> Result<PathBuf> {
        let mut secret_dir = self.config.secret_dir.clone();
        secret_dir.push(self.store.store_relative_path(&self.derivation)?);
        secret_dir.set_extension("");

        Ok(secret_dir)
    }

    /// Attempt to provision a secret using a specific backend
    /// returning the contents if successful.
    ///
    /// # Errors
    ///
    /// If the backend kind can't be instantiated.
    fn try_provision(
        &self,
        backend_kind: BackendKind,
        secret: &Secret,
    ) -> Result<Option<SecretContent>> {
        if !backend::validate_config(backend_kind, self.config) {
            return Ok(None);
        }

        Ok(backend::create(backend_kind, self.config)?.provision(secret))
    }

    /// Provision a secret. This method will enumerate
    /// backends until one is successful.
    ///
    /// # Errors
    ///
    /// If no backends can successfully decrypt the secret.
    pub fn provision<'s>(&self, secret: &'s Secret) -> Result<ProvisionedSecret<'s>> {
        debug!("provisioning secret: {:?}", secret);

        if let Some(backend_hint) = secret.backend_hint {
            debug!("found backend hint, trying backend {:?}", backend_hint);
            if let Some(content) = self.try_provision(backend_hint, secret)? {
                // TODO: verify hash
                return self.write_secret_content(secret, content);
            }
        }

        for backend_kind in BACKEND_KINDS {
            if let Some(backend_hint) = secret.backend_hint
                && backend_hint == backend_kind
            {
                continue;
            }

            if let Some(content) = self.try_provision(backend_kind, secret)? {
                // TODO: verify hash
                return self.write_secret_content(secret, content);
            }
        }

        Err(Error::NoSuccessfulBackends(secret.clone()))
    }

    /// Provision all the secrets required by the derivation. This method
    /// reads the "requiredSecrets" field of the derivation environment
    /// containing secret declarations.
    ///
    /// # Errors
    ///
    /// If the "requiredSecrets" field contains secret declarations that
    /// are unparsable.
    pub fn provision_all(&self) -> Result<()> {
        let required_secrets = self.required_secrets()?;

        let Some(required_secrets_serialized) = required_secrets else {
            debug!("derivation has no \"requiredSecrets\" field");
            return Ok(());
        };

        debug!("requiredSecrets: {}", required_secrets_serialized);

        for serialized_secret in required_secrets_serialized.split(' ') {
            let secret: Secret =
                serde_json::from_str(serialized_secret).map_err(Error::ParseSecret)?;
            self.provision(&secret)?;
        }

        Ok(())
    }

    /// Fetch the "requiredSecrets" field from the derivations environment.
    ///
    /// # Errors
    ///
    /// If nix throws an exception.
    pub fn required_secrets(&self) -> Result<Option<String>> {
        Ok(self
            .store
            .derivation_env_val(&self.derivation, "requiredSecrets")?)
    }

    fn write_secret_content<'s>(
        &self,
        secret: &'s Secret,
        content: SecretContent,
    ) -> Result<ProvisionedSecret<'s>> {
        let path = self.allocate_decrypted_file_path(secret)?;

        let mut file = match File::create(&path) {
            Ok(file) => file,
            Err(err) => {
                return Err(Error::CreateSecretFile { path, source: err });
            }
        };

        if let Err(err) = file.write_all(content.as_ref()) {
            return Err(Error::WriteSecret { path, source: err });
        }

        Ok(ProvisionedSecret {
            secret,
            content,
            path,
        })
    }

    fn allocate_decrypted_file_path(&self, secret: &Secret) -> Result<PathBuf> {
        let secret_dir = self.derivation_secret_directory()?;

        if !secret_dir.exists()
            && let Err(source) = std::fs::create_dir_all(&secret_dir)
        {
            return Err(Error::CreateDrvSecretDir {
                path: secret_dir,
                source,
            });
        }

        let mut secret_file = secret_dir;
        secret_file.push(&secret.name);

        Ok(secret_file)
    }
}
