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

const DEFAULT_KNOWLEDGE_LIMIT = 50;

export const knowledgeQueryKeys = {
  all: ["knowledge"] as const,
  list: (limit: number) =>
    [...knowledgeQueryKeys.all, "list", { limit }] as const,
};

export function useKnowledgeNodes(limit = DEFAULT_KNOWLEDGE_LIMIT) {
  return useQuery({
    queryKey: knowledgeQueryKeys.list(limit),
    queryFn: () => listKnowledgeNodes(limit),
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
