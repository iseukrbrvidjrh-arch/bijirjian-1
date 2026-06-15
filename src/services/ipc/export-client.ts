import { invoke } from "@tauri-apps/api/core";

import type { ExportRecordDto } from "@/types/export";

export function exportKnowledgeNode(knowledgeId: string) {
  return invoke<ExportRecordDto>("export_knowledge_node", {
    knowledgeId,
  });
}
