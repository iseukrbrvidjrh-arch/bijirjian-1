export type SourceType = "text" | "pdf";

export type InboxStatus =
  | "unprocessed"
  | "processed"
  | "dismissed"
  | "failed";

export interface SourceDto {
  id: string;
  workspaceId: string;
  sourceType: SourceType;
  rawContent: string;
  contentHash: string;
  metadataJson: string | null;
  inboxStatus: InboxStatus;
  capturedAt: string;
  processedAt: string | null;
  createdAt: string;
  updatedAt: string;
  deletedAt: string | null;
}

export interface InboxSourceListFilters {
  limit: number;
  query?: string;
}

export interface PdfSourceMetadata {
  originalFileName: string;
  fileSize: number;
  extractedTextLength: number;
  capturedVia: "pdf";
}
