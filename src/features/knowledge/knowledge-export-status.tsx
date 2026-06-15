import { useState } from "react";

import { Button } from "@/components/ui/button";
import { StatusBadge } from "@/components/ui/status-badge";
import {
  useExportKnowledgeNode,
  useLatestExportRecord,
} from "@/features/knowledge/knowledge-export-queries";
import {
  exportStatusLabel,
  formatDateTime,
  formatUiError,
} from "@/lib/display";

type CopyStatus = "idle" | "copied" | "failed";

export function KnowledgeExportStatus({
  knowledgeId,
}: {
  knowledgeId: string;
}) {
  const [copyStatus, setCopyStatus] = useState<CopyStatus>("idle");
  const latestExportQuery = useLatestExportRecord(knowledgeId, true);
  const exportMutation = useExportKnowledgeNode();
  const latestExport = latestExportQuery.data;

  function exportNode() {
    setCopyStatus("idle");
    exportMutation.reset();
    exportMutation.mutate(knowledgeId);
  }

  async function copyPath(exportPath: string) {
    setCopyStatus("idle");

    if (!navigator.clipboard?.writeText) {
      setCopyStatus("failed");
      return;
    }

    try {
      await navigator.clipboard.writeText(exportPath);
      setCopyStatus("copied");
    } catch {
      setCopyStatus("failed");
    }
  }

  return (
    <section className="mt-4 rounded-md border bg-muted/20 p-3">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h4 className="text-sm font-medium">导出到 Obsidian</h4>
        <Button
          size="sm"
          type="button"
          variant="outline"
          disabled={exportMutation.isPending}
          onClick={exportNode}
        >
          {exportMutation.isPending ? "正在导出…" : "导出"}
        </Button>
      </div>

      <div className="mt-3 text-xs" aria-live="polite">
        {exportMutation.isSuccess && (
          <p className="text-emerald-700">
            导出成功，最近导出信息已更新。
          </p>
        )}
        {exportMutation.error && (
          <p className="text-destructive" role="alert">
            导出失败：{formatUiError(exportMutation.error)}
          </p>
        )}
      </div>

      <div className="mt-3 border-t pt-3">
        {latestExportQuery.isPending && (
          <p className="text-xs text-muted-foreground">
            正在加载导出状态…
          </p>
        )}

        {latestExportQuery.error && (
          <div className="text-xs text-destructive" role="alert">
            <p>
              导出状态加载失败：
              {formatUiError(latestExportQuery.error)}
            </p>
            <Button
              className="mt-2"
              size="sm"
              type="button"
              variant="outline"
              onClick={() => void latestExportQuery.refetch()}
            >
              重试
            </Button>
          </div>
        )}

        {!latestExportQuery.isPending &&
          !latestExportQuery.error &&
          !latestExport && (
            <p className="text-xs text-muted-foreground">
              尚未导出
            </p>
          )}

        {!latestExportQuery.error && latestExport && (
          <div className="space-y-2 text-xs">
            <div className="flex flex-wrap items-center gap-2">
              <span className="font-medium">最近导出：</span>
              <StatusBadge
                tone={
                  latestExport.status === "succeeded" ? "green" : "red"
                }
              >
                {exportStatusLabel(latestExport.status)}
              </StatusBadge>
              <time
                className="text-muted-foreground"
                dateTime={latestExport.createdAt}
              >
                {formatDateTime(latestExport.createdAt)}
              </time>
              {latestExportQuery.isFetching && (
                <span className="text-muted-foreground">
                  正在刷新…
                </span>
              )}
            </div>

            {latestExport.status === "failed" &&
              latestExport.errorMessage && (
                <p className="text-destructive">
                  {formatUiError(
                    latestExport.errorMessage,
                    "导出失败，请检查 Obsidian 仓库路径。",
                  )}
                </p>
              )}

            {latestExport.exportPath && (
              <div>
                <p className="break-all font-mono text-muted-foreground">
                  {latestExport.exportPath}
                </p>
                <div className="mt-2 flex items-center gap-2">
                  <Button
                    size="sm"
                    type="button"
                    variant="outline"
                    onClick={() =>
                      void copyPath(latestExport.exportPath!)
                    }
                  >
                    复制路径
                  </Button>
                  {copyStatus === "copied" && (
                    <span className="text-emerald-700">
                      已复制路径
                    </span>
                  )}
                  {copyStatus === "failed" && (
                    <span className="text-destructive">
                      复制失败
                    </span>
                  )}
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    </section>
  );
}
