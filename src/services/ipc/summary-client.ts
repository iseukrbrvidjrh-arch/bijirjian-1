import { invoke } from "@tauri-apps/api/core";

import type { SourceSummaryDto } from "@/types/summary";

export async function summarizeSource(
  sourceId: string,
): Promise<SourceSummaryDto> {
  try {
    return await invoke<SourceSummaryDto>("summarize_source", {
      sourceId,
    });
  } catch (error) {
    throw error instanceof Error ? error : new Error(String(error));
  }
}
