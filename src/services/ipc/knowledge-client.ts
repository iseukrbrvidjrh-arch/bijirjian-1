import { invoke } from "@tauri-apps/api/core";

import type {
  CreateKnowledgeNodeInput,
  KnowledgeNodeDto,
} from "@/types/knowledge";

export function acceptKnowledgeNode(knowledgeId: string) {
  return invoke<KnowledgeNodeDto>("accept_knowledge_node", {
    knowledgeId,
  });
}

export function archiveKnowledgeNode(knowledgeId: string) {
  return invoke<KnowledgeNodeDto>("archive_knowledge_node", {
    knowledgeId,
  });
}

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
