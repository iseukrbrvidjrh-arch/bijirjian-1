export type KnowledgeType =
  | "concept"
  | "tool"
  | "project"
  | "question"
  | "solution"
  | "insight"
  | "resource"
  | "person";

export type KnowledgeStatus =
  | "proposed"
  | "accepted"
  | "archived";

export interface KnowledgeNodeDto {
  id: string;
  workspaceId: string;
  aiRunId: string | null;
  title: string;
  content: string;
  knowledgeType: KnowledgeType;
  status: KnowledgeStatus;
  createdAt: string;
  updatedAt: string;
  archivedAt: string | null;
}

export interface CreateKnowledgeNodeInput {
  title: string;
  content: string;
  knowledgeType: KnowledgeType;
}

export interface KnowledgeListFilters {
  limit: number;
  status?: KnowledgeStatus;
  knowledgeType?: KnowledgeType;
}
