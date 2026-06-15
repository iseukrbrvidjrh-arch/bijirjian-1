import { invoke } from "@tauri-apps/api/core";

import type { ExportRecordDto } from "@/types/export";

export function exportKnowledgeNode(knowledgeId: string) {
  return invoke<ExportRecordDto>("export_knowledge_node", {
    knowledgeId,
  });
}

export function getLatestExportRecordForKnowledge(
  knowledgeId: string,
) {
  return invoke<ExportRecordDto | null>(
    "get_latest_export_record_for_knowledge",
    {
      knowledgeId,
    },
  );
}
