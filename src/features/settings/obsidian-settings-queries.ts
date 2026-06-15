import {
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";

import {
  getObsidianSettings,
  saveObsidianSettings,
} from "@/services/ipc";

export const obsidianSettingsQueryKey = [
  "settings",
  "obsidian",
] as const;

export function useObsidianSettings() {
  return useQuery({
    queryKey: obsidianSettingsQueryKey,
    queryFn: getObsidianSettings,
  });
}

export function useSaveObsidianSettings() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: saveObsidianSettings,
    onSuccess: (settings) => {
      queryClient.setQueryData(obsidianSettingsQueryKey, settings);
    },
  });
}
