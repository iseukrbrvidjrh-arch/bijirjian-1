export type ExportStatus = "succeeded" | "failed";

export interface ExportRecordDto {
  id: string;
  workspaceId: string;
  knowledgeNodeId: string;
  exportPath: string | null;
  status: ExportStatus;
  errorMessage: string | null;
  createdAt: string;
}
