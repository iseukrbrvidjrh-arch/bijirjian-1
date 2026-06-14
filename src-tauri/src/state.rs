use crate::infrastructure::{
    ai::DefaultProviderRouter, database::Database, keyring::SystemCredentialStore,
};

pub struct AppState {
    pub database: Database,
    pub credential_store: SystemCredentialStore,
    pub provider_router: DefaultProviderRouter,
}

impl AppState {
    pub fn new(
        database: Database,
        credential_store: SystemCredentialStore,
        provider_router: DefaultProviderRouter,
    ) -> Self {
        Self {
            database,
            credential_store,
            provider_router,
        }
    }
}
