import { useMutation } from "@tanstack/react-query";

import { exportKnowledgeNode } from "@/services/ipc";

export function useExportKnowledgeNode() {
  return useMutation({
    mutationFn: exportKnowledgeNode,
  });
}
