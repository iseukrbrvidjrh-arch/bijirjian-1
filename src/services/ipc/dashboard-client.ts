import { invoke } from "@tauri-apps/api/core";

import type { DashboardSummaryDto } from "@/types/dashboard";

export function getDashboardSummary() {
  return invoke<DashboardSummaryDto>("get_dashboard_summary");
}
