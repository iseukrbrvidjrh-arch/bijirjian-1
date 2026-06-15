import type { KnowledgeNodeDto } from "@/types/knowledge";
import type { SourceDto } from "@/types/source";
import type { LatestSourceSummaryDto } from "@/types/summary";

export interface SourceDetailDto {
  source: SourceDto;
  latestSummary: LatestSourceSummaryDto | null;
  relatedKnowledge: KnowledgeNodeDto | null;
}
