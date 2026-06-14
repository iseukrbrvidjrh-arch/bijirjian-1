export interface PromptVersionDto {
  id: string;
  promptId: string;
  version: number;
  promptContent: string;
  createdAt: string;
}

export interface DefaultPromptDto {
  id: string;
  promptKey: string;
  name: string;
  description: string | null;
  activeVersionId: string;
  activeVersion: PromptVersionDto;
  createdAt: string;
  updatedAt: string;
}
