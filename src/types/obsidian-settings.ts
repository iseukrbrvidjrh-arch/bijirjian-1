export interface ObsidianSettingsDto {
  workspaceId: string;
  vaultPath: string;
  hasObsidianDirectory: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface SaveObsidianSettingsInput {
  vaultPath: string;
}
