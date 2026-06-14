import {
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";

import {
  getAiProviderSettings,
  saveAiProviderSettings,
  testAiProviderConnection,
} from "@/services/ipc";

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
