import { invoke } from "@tauri-apps/api/core";

import type { SourceDto } from "@/types/source";

export async function captureTextSource(
  rawContent: string,
): Promise<SourceDto> {
  return invokeCommand<SourceDto>("capture_text_source", { rawContent });
}

export async function listInboxSources(
  limit?: number,
): Promise<SourceDto[]> {
  return invokeCommand<SourceDto[]>("list_inbox_sources", { limit });
}

async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw error instanceof Error ? error : new Error(String(error));
  }
}
