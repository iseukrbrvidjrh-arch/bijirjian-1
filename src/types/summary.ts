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
