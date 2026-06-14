export type AiProviderType = "deepseek";
export type AiProviderModel =
  | "deepseek-v4-flash"
  | "deepseek-v4-pro";

export interface AiProviderSettingsDto {
  providerType: AiProviderType;
  defaultModel: AiProviderModel;
  hasApiKey: boolean;
  updatedAt: string;
}

export interface SaveAiProviderSettingsInput {
  providerType: AiProviderType;
  defaultModel: AiProviderModel;
  apiKey?: string;
}

export interface ProviderConnectionResultDto {
  providerType: AiProviderType;
  message: string;
}
