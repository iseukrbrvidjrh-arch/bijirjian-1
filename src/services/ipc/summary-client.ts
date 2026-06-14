import { invoke } from "@tauri-apps/api/core";

import type {
  LatestSourceSummaryDto,
  SourceSummaryDto,
} from "@/types/summary";

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

export async function getLatestSourceSummary(
  sourceId: string,
): Promise<LatestSourceSummaryDto | null> {
  try {
    return await invoke<LatestSourceSummaryDto | null>(
      "get_latest_source_summary",
      {
        sourceId,
      },
    );
  } catch (error) {
    throw error instanceof Error ? error : new Error(String(error));
  }
}
