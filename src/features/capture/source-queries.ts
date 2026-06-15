import {
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";

import {
  capturePdfSource,
  captureTextSource,
  getSourceDetail,
  listInboxSources,
  markSourceDismissed,
  markSourceProcessed,
} from "@/services/ipc";
import type { InboxSourceListFilters } from "@/types/source";

const DEFAULT_INBOX_LIMIT = 50;

export const sourceQueryKeys = {
  all: ["sources"] as const,
  inbox: () => [...sourceQueryKeys.all, "inbox"] as const,
  inboxList: ({ limit, query }: InboxSourceListFilters) =>
    [...sourceQueryKeys.inbox(), { limit, query }] as const,
  detail: (sourceId: string) =>
    [...sourceQueryKeys.all, sourceId, "detail"] as const,
};

export function useInboxSources({
  limit = DEFAULT_INBOX_LIMIT,
  query,
}: Partial<InboxSourceListFilters> = {}) {
  const filters = {
    limit,
    query: query?.trim() || undefined,
  };

  return useQuery({
    queryKey: sourceQueryKeys.inboxList(filters),
    queryFn: () => listInboxSources(filters),
  });
}

export function useSourceDetail(sourceId: string, enabled = true) {
  return useQuery({
    queryKey: sourceQueryKeys.detail(sourceId),
    queryFn: () => getSourceDetail(sourceId),
    enabled,
  });
}

export function useCaptureTextSource() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: captureTextSource,
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: sourceQueryKeys.inbox(),
      });
    },
  });
}

export function useCapturePdfSource() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: capturePdfSource,
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: sourceQueryKeys.inbox(),
      });
    },
  });
}

export function useMarkSourceProcessed() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: markSourceProcessed,
    onSuccess: async (_source, sourceId) => {
      await Promise.all([
        queryClient.invalidateQueries({
          queryKey: sourceQueryKeys.inbox(),
        }),
        queryClient.invalidateQueries({
          queryKey: sourceQueryKeys.detail(sourceId),
        }),
      ]);
    },
  });
}

export function useMarkSourceDismissed() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: markSourceDismissed,
    onSuccess: async (_source, sourceId) => {
      await Promise.all([
        queryClient.invalidateQueries({
          queryKey: sourceQueryKeys.inbox(),
        }),
        queryClient.invalidateQueries({
          queryKey: sourceQueryKeys.detail(sourceId),
        }),
      ]);
    },
  });
}
