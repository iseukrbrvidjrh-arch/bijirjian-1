import { invoke } from "@tauri-apps/api/core";

import type {
  DefaultPromptDto,
  PromptVersionDto,
} from "@/types/prompt";

export function getDefaultPrompt() {
  return invoke<DefaultPromptDto>("get_default_prompt");
}

export function listPromptVersions() {
  return invoke<PromptVersionDto[]>("list_prompt_versions");
}

export function createPromptVersion(promptContent: string) {
  return invoke<PromptVersionDto>("create_prompt_version", {
    promptContent,
  });
}

export function setActivePromptVersion(versionId: string) {
  return invoke<DefaultPromptDto>("set_active_prompt_version", {
    versionId,
  });
}
