import { useMutation } from "@tanstack/react-query";

import { summarizeSource } from "@/services/ipc";

export function useSummarizeSource() {
  return useMutation({
    mutationFn: summarizeSource,
  });
}
