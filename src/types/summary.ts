import type {
  AiProviderModel,
  AiProviderType,
} from "@/types/ai-provider";

export interface SourceSummaryDto {
  sourceId: string;
  summary: string;
  providerType: AiProviderType;
  model: AiProviderModel;
  promptVersionId: string;
  promptVersion: number;
}

export type AiRunStatus = "succeeded" | "failed";

export interface LatestSourceSummaryDto {
  runId: string;
  sourceId: string;
  summary: string | null;
  status: AiRunStatus;
  errorMessage: string | null;
  providerType: AiProviderType | null;
  model: AiProviderModel | null;
  promptVersionId: string | null;
  promptVersion: number | null;
  createdAt: string;
  completedAt: string;
}
