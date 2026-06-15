import type { KnowledgeNodeDto } from "@/types/knowledge";
import type { SourceDto } from "@/types/source";

export interface DashboardSummaryDto {
  inboxUnprocessedCount: number;
  knowledgeTotalCount: number;
  proposedKnowledgeCount: number;
  acceptedKnowledgeCount: number;
  archivedKnowledgeCount: number;
  recentKnowledge: KnowledgeNodeDto[];
  recentInboxSources: SourceDto[];
  obsidianVaultConfigured: boolean;
}
