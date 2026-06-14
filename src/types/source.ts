export type SourceType = "text";

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
