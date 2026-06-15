import type { PdfSourceMetadata } from "@/types/source";

export function parsePdfSourceMetadata(
  metadataJson: string | null,
): PdfSourceMetadata | null {
  if (!metadataJson) {
    return null;
  }

  try {
    const value: unknown = JSON.parse(metadataJson);
    if (
      typeof value !== "object" ||
      value === null ||
      !("originalFileName" in value) ||
      !("fileSize" in value) ||
      !("extractedTextLength" in value) ||
      !("capturedVia" in value)
    ) {
      return null;
    }

    const metadata = value as Record<string, unknown>;
    if (
      typeof metadata.originalFileName !== "string" ||
      typeof metadata.fileSize !== "number" ||
      typeof metadata.extractedTextLength !== "number" ||
      metadata.capturedVia !== "pdf"
    ) {
      return null;
    }

    return {
      originalFileName: metadata.originalFileName,
      fileSize: metadata.fileSize,
      extractedTextLength: metadata.extractedTextLength,
      capturedVia: "pdf",
    };
  } catch {
    return null;
  }
}

export function formatFileSize(bytes: number) {
  if (!Number.isFinite(bytes) || bytes < 0) {
    return "Unknown size";
  }

  if (bytes < 1024) {
    return `${bytes} B`;
  }

  const units = ["KiB", "MiB", "GiB"];
  let value = bytes / 1024;
  let unit = units[0];

  for (let index = 1; index < units.length && value >= 1024; index += 1) {
    value /= 1024;
    unit = units[index];
  }

  return `${value.toFixed(value >= 10 ? 0 : 1)} ${unit}`;
}
