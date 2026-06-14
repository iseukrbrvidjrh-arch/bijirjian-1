#[cfg(target_os = "macos")]
use std::sync::Arc;

#[cfg(target_os = "macos")]
use apple_native_keyring_store::keychain::Store;
#[cfg(target_os = "macos")]
use keyring_core::{CredentialStore as NativeCredentialStore, Entry, Error as KeyringError};

use crate::{
    domain::{ports::CredentialStore, ProviderType},
    error::AppError,
};

const KEYRING_SERVICE: &str = "com.secondbrain.os";

pub struct SystemCredentialStore {
    #[cfg(target_os = "macos")]
    store: Arc<NativeCredentialStore>,
}

impl SystemCredentialStore {
    pub fn new() -> Result<Self, AppError> {
        #[cfg(target_os = "macos")]
        {
            let store: Arc<NativeCredentialStore> = Store::new().map_err(keyring_error)?;
            Ok(Self { store })
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err(AppError::Credential(
                "no supported system credential store is available on this platform".to_owned(),
            ))
        }
    }

    #[cfg(target_os = "macos")]
    fn entry(&self, provider_type: ProviderType) -> Result<Entry, AppError> {
        self.store
            .build(
                KEYRING_SERVICE,
                &format!("ai-provider:{}:api-key", provider_type.as_str()),
                None,
            )
            .map_err(keyring_error)
    }
}

impl CredentialStore for SystemCredentialStore {
    fn get_api_key(&self, provider_type: ProviderType) -> Result<Option<String>, AppError> {
        #[cfg(target_os = "macos")]
        {
            match self.entry(provider_type)?.get_password() {
                Ok(api_key) => Ok(Some(api_key)),
                Err(KeyringError::NoEntry) => Ok(None),
                Err(error) => Err(keyring_error(error)),
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = provider_type;
            Err(AppError::Credential(
                "no supported system credential store is available on this platform".to_owned(),
            ))
        }
    }

    fn set_api_key(&self, provider_type: ProviderType, api_key: &str) -> Result<(), AppError> {
        #[cfg(target_os = "macos")]
        {
            self.entry(provider_type)?
                .set_password(api_key)
                .map_err(keyring_error)
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = (provider_type, api_key);
            Err(AppError::Credential(
                "no supported system credential store is available on this platform".to_owned(),
            ))
        }
    }
}

#[cfg(target_os = "macos")]
fn keyring_error(error: KeyringError) -> AppError {
    AppError::Credential(format!("system Keychain operation failed: {error}"))
}
