import { useQuery } from "@tanstack/react-query";

import { getDashboardSummary } from "@/services/ipc";

export const dashboardQueryKeys = {
  summary: ["dashboard", "summary"] as const,
};

export function useDashboardSummary() {
  return useQuery({
    queryKey: dashboardQueryKeys.summary,
    queryFn: getDashboardSummary,
    staleTime: 0,
    refetchOnMount: "always",
  });
}
