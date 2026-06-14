import { invoke } from "@tauri-apps/api/core";

import type {
  AiProviderSettingsDto,
  ProviderConnectionResultDto,
  SaveAiProviderSettingsInput,
} from "@/types/ai-provider";

export function getAiProviderSettings() {
  return invoke<AiProviderSettingsDto | null>("get_ai_provider_settings");
}

export function saveAiProviderSettings({
  providerType,
  defaultModel,
  apiKey,
}: SaveAiProviderSettingsInput) {
  return invoke<AiProviderSettingsDto>("save_ai_provider_settings", {
    providerType,
    defaultModel,
    apiKey,
  });
}

export function testAiProviderConnection() {
  return invoke<ProviderConnectionResultDto>(
    "test_ai_provider_connection",
  );
}
