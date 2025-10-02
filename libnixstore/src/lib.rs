#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod error;

use error::{Error, Result};
use std::path::Path;
use std::sync::Once;
use tracing::instrument;

static INITIALIZE_LIBNIXSTORE: Once = Once::new();

#[cxx::bridge(namespace = "wrap")]
mod ffi {
    enum NixErrorTag {
        GetVersion,
        StorePath,
        EnvKeyDoesNotExist,
    }

    unsafe extern "C++" {
        include!("libnixstore/include/nix-wrap.hh");

        fn init_lib_nix_store() -> Result<()>;

        type StorePath;

        type LocalStore;
        fn new_local_store() -> Result<UniquePtr<LocalStore>>;
        fn get_version(self: &LocalStore) -> Result<String>;
        fn parse_store_path(self: &LocalStore, path: &[u8]) -> Result<SharedPtr<StorePath>>;
        fn get_derivation_env_val(
            self: &LocalStore,
            path: SharedPtr<StorePath>,
            key: &str,
        ) -> Result<String>;
        fn get_derivation_name(self: &LocalStore, path: SharedPtr<StorePath>) -> Result<String>;
        fn get_store_relative_path(self: &LocalStore, path: SharedPtr<StorePath>)
        -> Result<String>;
    }
}

pub struct Store(cxx::UniquePtr<ffi::LocalStore>);
pub struct StorePath(cxx::SharedPtr<ffi::StorePath>);

impl StorePath {
    fn inner(&self) -> cxx::SharedPtr<ffi::StorePath> {
        self.0.clone()
    }
}

impl Store {
    /// Create a new instance of `Store` which is a handle on the local
    /// nix store.
    ///
    /// # Errors
    ///
    /// If libnixstore can't be initialized, or if the local
    /// store can't be opened
    ///
    /// # Panics
    ///
    /// If `initLibStore()` from libnixstore throws an exception
    #[tracing::instrument]
    pub fn new() -> Result<Self> {
        INITIALIZE_LIBNIXSTORE.call_once(|| {
            ffi::init_lib_nix_store().expect("initLibStore() failed with an exception");
        });

        Ok(Self(ffi::new_local_store()?))
    }

    /// Get the nix version
    ///
    /// # Errors
    ///
    /// If any exceptions are thrown from nix's `Store::getVersion()`
    #[instrument(skip_all)]
    pub fn version(&self) -> Result<String> {
        Ok(self.0.get_version()?)
    }

    /// Parse and validate a store path
    ///
    /// # Errors
    ///
    /// If the store path is not a path in the store, if the
    /// path doesn't reference a valid store object, or if nix
    /// throws an exception for some other reason.
    #[instrument(skip_all)]
    pub fn parse_store_path<T: AsRef<Path>>(&self, path: T) -> Result<StorePath> {
        let path = self
            .0
            .parse_store_path(path.as_ref().as_os_str().as_encoded_bytes())?;

        Ok(StorePath(path))
    }

    /// Fetch a value from the derivation environment
    /// referenced by the store path.
    ///
    /// # Errors
    ///
    /// If nix throws an exception.
    #[instrument(skip_all)]
    pub fn derivation_env_val(&self, drv_path: &StorePath, key: &str) -> Result<Option<String>> {
        let path = drv_path.inner();
        let result: Result<String> = self
            .0
            .get_derivation_env_val(path, key)
            .map_err(Into::<Error>::into);

        match result {
            Ok(value) => Ok(Some(value)),
            Err(Error::EnvKeyDoesNotExist(_)) => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// Fetch the name of a derivation.
    ///
    /// # Errors
    ///
    /// If nix throws an exception.
    #[instrument(skip_all)]
    pub fn derivation_name(&self, drv_path: &StorePath) -> Result<String> {
        let path = drv_path.inner();
        Ok(self.0.get_derivation_name(path)?)
    }

    /// Fetch the "store relative" path of the object
    /// referenced by `store_path`
    ///
    /// # Errors
    ///
    /// If nix throws an exception.
    #[instrument(skip_all)]
    pub fn store_relative_path(&self, store_path: &StorePath) -> Result<String> {
        let path = store_path.inner();
        Ok(self.0.get_store_relative_path(path)?)
    }
}

#[cfg(test)]
mod tests {
    use super::Store;
    use crate::error::Error;

    #[test]
    fn create_local_store() {
        let res = Store::new();
        assert!(res.is_ok());
    }

    #[test]
    fn store_get_version() {
        let store = Store::new().expect("Store::new");

        let version = store.version().expect("store.version");

        assert!(version.contains('.'));
    }

    #[test]
    fn store_parse_invalid_path() {
        let store = Store::new().expect("Store::new");

        let parse = store
            .parse_store_path("/nix/store/aaaaaaaaaaaaaaaaaaaaaaad0xxc253d-dwm-status-1.10.0.drv");

        assert!(matches!(parse, Err(
            Error::StorePath(what)
            ) if what.contains("is not valid")));
    }

    #[test]
    fn store_parse_broken_path() {
        let store = Store::new().expect("Store::new");

        let parse = store
            .parse_store_path("/nix/store/2qwfcp-----------------d0xxc253d-dwm-status-1.10.0.drv");

        assert!(matches!(parse, Err(
            Error::StorePath(what)
            ) if what.contains("contains illegal base-32 character")));
    }

    #[test]
    fn store_parse_path_external_to_store() {
        let store = Store::new().expect("Store::new");

        let parse = store.parse_store_path("/not-nix/not-store/meow");

        assert!(matches!(parse, Err(
            Error::StorePath(what)
            ) if what.contains("is not in the Nix store")));
    }

    #[test]
    fn store_read_non_existant_derivation_val() {
        let store = Store::new().expect("Store::new");

        let parse = store
            .parse_store_path("/nix/store/2qwfcpv54pb5l7nbyzg16rbd0xxc253d-dwm-status-1.10.0.drv")
            .expect("store.parse_store_path");

        let env_val = store.derivation_env_val(&parse, "meow");

        assert!(matches!(env_val, Ok(None)));
    }

    #[test]
    fn store_read_derivation_val() {
        let store = Store::new().expect("Store::new");

        let parse = store
            .parse_store_path("/nix/store/2qwfcpv54pb5l7nbyzg16rbd0xxc253d-dwm-status-1.10.0.drv")
            .expect("store.parse_store_path");

        let env_val = store
            .derivation_env_val(&parse, "cargoDeps")
            .expect("store.derivation_env_val");

        assert!(matches!(
            env_val,
            Some(val) if val.contains(
                "/nix/store/64phcl7q29iy5zrabw5088xvx2ad7qia-dwm-status-1.10.0-vendor"
            )
        ));
    }

    #[test]
    fn store_read_derivation_name() {
        let store = Store::new().expect("Store::new");

        let parse = store
            .parse_store_path("/nix/store/2qwfcpv54pb5l7nbyzg16rbd0xxc253d-dwm-status-1.10.0.drv")
            .expect("store.parse_store_path");

        let name = store
            .derivation_name(&parse)
            .expect("store.derivation_name");

        assert!(name.contains("dwm-status-1.10.0"));
    }

    #[test]
    fn store_path_relative_name() {
        let store = Store::new().expect("Store::new");

        let parse = store
            .parse_store_path("/nix/store/2qwfcpv54pb5l7nbyzg16rbd0xxc253d-dwm-status-1.10.0.drv")
            .expect("store.parse_store_path");

        let path = store
            .store_relative_path(&parse)
            .expect("store.derivation_name");

        assert_eq!(
            path,
            "2qwfcpv54pb5l7nbyzg16rbd0xxc253d-dwm-status-1.10.0.drv"
        );
    }
}
