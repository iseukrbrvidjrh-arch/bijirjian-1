import { invoke } from "@tauri-apps/api/core";

import type {
  CreateKnowledgeNodeInput,
  KnowledgeNodeDto,
} from "@/types/knowledge";

export function createKnowledgeNode({
  title,
  content,
  knowledgeType,
}: CreateKnowledgeNodeInput) {
  return invoke<KnowledgeNodeDto>("create_knowledge_node", {
    title,
    content,
    knowledgeType,
  });
}

export function listKnowledgeNodes(limit = 50) {
  return invoke<KnowledgeNodeDto[]>("list_knowledge_nodes", {
    limit,
  });
}

export function createKnowledgeDraftFromLatestSummary(
  sourceId: string,
) {
  return invoke<KnowledgeNodeDto>(
    "create_knowledge_draft_from_latest_summary",
    {
      sourceId,
    },
  );
}
