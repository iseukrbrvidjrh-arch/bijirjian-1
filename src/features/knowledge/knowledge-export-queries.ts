import {
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";

import {
  exportKnowledgeNode,
  getLatestExportRecordForKnowledge,
} from "@/services/ipc";

export const knowledgeExportQueryKeys = {
  latest: (knowledgeId: string) =>
    ["knowledge", knowledgeId, "latest-export"] as const,
};

export function useLatestExportRecord(
  knowledgeId: string,
  enabled: boolean,
) {
  return useQuery({
    queryKey: knowledgeExportQueryKeys.latest(knowledgeId),
    queryFn: () => getLatestExportRecordForKnowledge(knowledgeId),
    enabled,
  });
}

export function useExportKnowledgeNode() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: exportKnowledgeNode,
    onSettled: async (_data, _error, knowledgeId) => {
      await queryClient.invalidateQueries({
        queryKey: knowledgeExportQueryKeys.latest(knowledgeId),
      });
    },
  });
}
