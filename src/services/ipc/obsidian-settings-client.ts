import { invoke } from "@tauri-apps/api/core";

import type {
  ObsidianSettingsDto,
  SaveObsidianSettingsInput,
} from "@/types/obsidian-settings";

export function getObsidianSettings() {
  return invoke<ObsidianSettingsDto | null>("get_obsidian_settings");
}

export function saveObsidianSettings({
  vaultPath,
}: SaveObsidianSettingsInput) {
  return invoke<ObsidianSettingsDto>("save_obsidian_settings", {
    vaultPath,
  });
}
