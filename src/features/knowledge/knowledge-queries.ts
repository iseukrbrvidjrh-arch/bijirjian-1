import {
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";

import {
  acceptKnowledgeNode,
  archiveKnowledgeNode,
  createKnowledgeDraftFromLatestSummary,
  createKnowledgeNode,
  listKnowledgeNodes,
} from "@/services/ipc";
import type { KnowledgeListFilters } from "@/types/knowledge";

export const knowledgeQueryKeys = {
  all: ["knowledge"] as const,
  list: (filters: KnowledgeListFilters) =>
    [
      ...knowledgeQueryKeys.all,
      "list",
      {
        limit: filters.limit,
        status: filters.status,
        knowledgeType: filters.knowledgeType,
      },
    ] as const,
};

export function useKnowledgeNodes(filters: KnowledgeListFilters) {
  return useQuery({
    queryKey: knowledgeQueryKeys.list(filters),
    queryFn: () => listKnowledgeNodes(filters),
  });
}

export function useAcceptKnowledgeNode() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: acceptKnowledgeNode,
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: knowledgeQueryKeys.all,
      });
    },
  });
}

export function useArchiveKnowledgeNode() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: archiveKnowledgeNode,
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: knowledgeQueryKeys.all,
      });
    },
  });
}

export function useCreateKnowledgeDraftFromLatestSummary() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: createKnowledgeDraftFromLatestSummary,
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: knowledgeQueryKeys.all,
      });
    },
  });
}

export function useCreateKnowledgeNode() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: createKnowledgeNode,
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: knowledgeQueryKeys.all,
      });
    },
  });
}
