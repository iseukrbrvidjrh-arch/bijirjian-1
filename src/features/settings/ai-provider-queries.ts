import {
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";

import {
  getAiProviderSettings,
  listAiProviderModels,
  saveAiProviderSettings,
  testAiProviderConnection,
} from "@/services/ipc";
import type { ListAiProviderModelsInput } from "@/types/ai-provider";

export const aiProviderQueryKey = ["settings", "ai-provider"] as const;

export function useAiProviderSettings() {
  return useQuery({
    queryKey: aiProviderQueryKey,
    queryFn: getAiProviderSettings,
  });
}

export function useSaveAiProviderSettings() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: saveAiProviderSettings,
    onSuccess: (settings) => {
      queryClient.setQueryData(aiProviderQueryKey, settings);
    },
  });
}

export function useTestAiProviderConnection() {
  return useMutation({
    mutationFn: testAiProviderConnection,
  });
}

export function useListAiProviderModels() {
  return useMutation({
    mutationFn: (input: ListAiProviderModelsInput) =>
      listAiProviderModels(input),
  });
}
