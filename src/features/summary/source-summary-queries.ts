import {
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";

import {
  getLatestSourceSummary,
  summarizeSource,
} from "@/services/ipc";
import { sourceQueryKeys } from "@/features/capture/source-queries";

export const sourceSummaryQueryKeys = {
  latest: (sourceId: string) =>
    ["sources", sourceId, "latest-summary"] as const,
};

export function useLatestSourceSummary(sourceId: string) {
  return useQuery({
    queryKey: sourceSummaryQueryKeys.latest(sourceId),
    queryFn: () => getLatestSourceSummary(sourceId),
  });
}

export function useSummarizeSource() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: summarizeSource,
    onSettled: async (_data, _error, sourceId) => {
      await Promise.all([
        queryClient.invalidateQueries({
          queryKey: sourceSummaryQueryKeys.latest(sourceId),
        }),
        queryClient.invalidateQueries({
          queryKey: sourceQueryKeys.detail(sourceId),
        }),
      ]);
    },
  });
}
