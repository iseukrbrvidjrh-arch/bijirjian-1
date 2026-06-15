export type AiProviderType = "deepseek" | "qwen" | "openai" | "gemini";

export type AiProviderModel = string;

export type ModelSource = "remote" | "builtin";

export interface ProviderModelInfoDto {
  id: string;
  label: string;
  providerType: AiProviderType;
  source: ModelSource;
}

export interface ProviderModelListDto {
  models: ProviderModelInfoDto[];
  usedFallback: boolean;
  fallbackReason: string | null;
}

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

export interface ListAiProviderModelsInput {
  providerType: AiProviderType;
  apiKey?: string;
}

export const providerDefaultModels: Record<AiProviderType, AiProviderModel> = {
  deepseek: "deepseek-v4-flash",
  qwen: "qwen-plus",
  openai: "gpt-4o",
  gemini: "gemini-2.5-flash",
};

export const builtinProviderModels: Record<
  AiProviderType,
  readonly ProviderModelInfoDto[]
> = {
  deepseek: [
    { id: "deepseek-v4-flash", label: "DeepSeek V4 Flash", providerType: "deepseek", source: "builtin" },
    { id: "deepseek-v4-pro", label: "DeepSeek V4 Pro", providerType: "deepseek", source: "builtin" },
  ],
  qwen: [
    { id: "qwen-plus", label: "Qwen Plus", providerType: "qwen", source: "builtin" },
    { id: "qwen-turbo", label: "Qwen Turbo", providerType: "qwen", source: "builtin" },
    { id: "qwen-max", label: "Qwen Max", providerType: "qwen", source: "builtin" },
    { id: "qwen-flash", label: "Qwen Flash", providerType: "qwen", source: "builtin" },
    { id: "qwen-long", label: "Qwen Long", providerType: "qwen", source: "builtin" },
    { id: "qwen-coder-plus", label: "Qwen Coder Plus", providerType: "qwen", source: "builtin" },
  ],
  openai: [
    { id: "gpt-4.1", label: "GPT-4.1", providerType: "openai", source: "builtin" },
    { id: "gpt-4o", label: "GPT-4o", providerType: "openai", source: "builtin" },
    { id: "gpt-4o-mini", label: "GPT-4o Mini", providerType: "openai", source: "builtin" },
    { id: "gpt-4.1-mini", label: "GPT-4.1 Mini", providerType: "openai", source: "builtin" },
    { id: "gpt-4-turbo", label: "GPT-4 Turbo", providerType: "openai", source: "builtin" },
    { id: "o3-mini", label: "o3-mini", providerType: "openai", source: "builtin" },
  ],
  gemini: [
    { id: "gemini-2.5-flash", label: "Gemini 2.5 Flash", providerType: "gemini", source: "builtin" },
    { id: "gemini-2.5-pro", label: "Gemini 2.5 Pro", providerType: "gemini", source: "builtin" },
    { id: "gemini-2.0-flash", label: "Gemini 2.0 Flash", providerType: "gemini", source: "builtin" },
    { id: "gemini-1.5-flash", label: "Gemini 1.5 Flash", providerType: "gemini", source: "builtin" },
    { id: "gemini-1.5-pro", label: "Gemini 1.5 Pro", providerType: "gemini", source: "builtin" },
  ],
};
