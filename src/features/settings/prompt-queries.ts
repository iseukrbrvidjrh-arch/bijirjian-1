import {
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";

import {
  createPromptVersion,
  getDefaultPrompt,
  listPromptVersions,
  setActivePromptVersion,
} from "@/services/ipc";

export const defaultPromptQueryKey = [
  "settings",
  "prompt",
  "default",
] as const;
export const promptVersionsQueryKey = [
  "settings",
  "prompt",
  "versions",
] as const;

export function useDefaultPrompt() {
  return useQuery({
    queryKey: defaultPromptQueryKey,
    queryFn: getDefaultPrompt,
  });
}

export function usePromptVersions() {
  return useQuery({
    queryKey: promptVersionsQueryKey,
    queryFn: listPromptVersions,
  });
}

export function useCreatePromptVersion() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: createPromptVersion,
    onSuccess: () =>
      queryClient.invalidateQueries({
        queryKey: promptVersionsQueryKey,
      }),
  });
}

export function useSetActivePromptVersion() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: setActivePromptVersion,
    onSuccess: (prompt) => {
      queryClient.setQueryData(defaultPromptQueryKey, prompt);
      queryClient.invalidateQueries({
        queryKey: promptVersionsQueryKey,
      });
    },
  });
}
