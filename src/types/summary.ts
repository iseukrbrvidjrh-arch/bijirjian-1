import type { AiProviderType } from "@/types/ai-provider";

export type AiRunStatus = "succeeded" | "failed";

export interface SourceSummaryDto {
  sourceId: string;
  summary: string;
  providerType: AiProviderType;
  model: string;
  promptVersionId: string;
  promptVersion: number;
}

export interface LatestSourceSummaryDto {
  runId: string;
  sourceId: string;
  summary: string | null;
  status: AiRunStatus;
  errorMessage: string | null;
  providerType: AiProviderType | null;
  model: string | null;
  promptVersionId: string | null;
  promptVersion: number | null;
  createdAt: string;
  completedAt: string;
}
