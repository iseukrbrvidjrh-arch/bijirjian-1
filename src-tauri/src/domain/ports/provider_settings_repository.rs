use crate::{
    domain::{ProviderModel, ProviderSettings, ProviderType},
    error::AppError,
};

pub trait ProviderSettingsRepository: Send + Sync {
    fn get_provider_settings(&self) -> Result<Option<ProviderSettings>, AppError>;

    fn save_provider_settings(
        &self,
        provider_type: ProviderType,
        default_model: ProviderModel,
    ) -> Result<ProviderSettings, AppError>;
}
