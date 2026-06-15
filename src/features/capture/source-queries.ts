import {
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";

import {
  captureTextSource,
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

export function useMarkSourceProcessed() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: markSourceProcessed,
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: sourceQueryKeys.inbox(),
      });
    },
  });
}

export function useMarkSourceDismissed() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: markSourceDismissed,
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: sourceQueryKeys.inbox(),
      });
    },
  });
}
