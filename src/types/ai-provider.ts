export type AiProviderType = "deepseek";

export interface AiProviderSettingsDto {
  providerType: AiProviderType;
  hasApiKey: boolean;
  updatedAt: string;
}

export interface SaveAiProviderSettingsInput {
  providerType: AiProviderType;
  apiKey?: string;
}

export interface ProviderConnectionResultDto {
  providerType: AiProviderType;
  message: string;
}
