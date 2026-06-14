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

const DEFAULT_INBOX_LIMIT = 50;

export const sourceQueryKeys = {
  all: ["sources"] as const,
  inbox: () => [...sourceQueryKeys.all, "inbox"] as const,
  inboxList: (limit: number) =>
    [...sourceQueryKeys.inbox(), { limit }] as const,
};

export function useInboxSources(limit = DEFAULT_INBOX_LIMIT) {
  return useQuery({
    queryKey: sourceQueryKeys.inboxList(limit),
    queryFn: () => listInboxSources(limit),
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
