import { useState } from "react";

import { Button } from "@/components/ui/button";
import {
  useExportKnowledgeNode,
  useLatestExportRecord,
} from "@/features/knowledge/knowledge-export-queries";

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
        <h4 className="text-sm font-medium">Obsidian export</h4>
        <Button
          size="sm"
          type="button"
          variant="outline"
          disabled={exportMutation.isPending}
          onClick={exportNode}
        >
          {exportMutation.isPending ? "Exporting…" : "Export"}
        </Button>
      </div>

      <div className="mt-3 text-xs" aria-live="polite">
        {exportMutation.isSuccess && (
          <p className="text-emerald-700 dark:text-emerald-300">
            Export succeeded. Latest export details are up to date.
          </p>
        )}
        {exportMutation.error && (
          <p className="text-destructive" role="alert">
            Export failed: {exportMutation.error.message}
          </p>
        )}
      </div>

      <div className="mt-3 border-t pt-3">
        {latestExportQuery.isPending && (
          <p className="text-xs text-muted-foreground">
            Loading export status…
          </p>
        )}

        {latestExportQuery.error && (
          <div className="text-xs text-destructive" role="alert">
            <p>
              Could not load export status:{" "}
              {latestExportQuery.error.message}
            </p>
            <Button
              className="mt-2"
              size="sm"
              type="button"
              variant="outline"
              onClick={() => void latestExportQuery.refetch()}
            >
              Retry
            </Button>
          </div>
        )}

        {!latestExportQuery.isPending &&
          !latestExportQuery.error &&
          !latestExport && (
            <p className="text-xs text-muted-foreground">
              Never exported
            </p>
          )}

        {!latestExportQuery.error && latestExport && (
          <div className="space-y-2 text-xs">
            <div className="flex flex-wrap items-center gap-2">
              <span
                className={
                  latestExport.status === "succeeded"
                    ? "font-medium text-emerald-700 dark:text-emerald-300"
                    : "font-medium text-destructive"
                }
              >
                Last export:{" "}
                {latestExport.status === "succeeded"
                  ? "Succeeded"
                  : "Failed"}
              </span>
              <time
                className="text-muted-foreground"
                dateTime={latestExport.createdAt}
              >
                {formatTimestamp(latestExport.createdAt)}
              </time>
              {latestExportQuery.isFetching && (
                <span className="text-muted-foreground">
                  Refreshing…
                </span>
              )}
            </div>

            {latestExport.status === "failed" &&
              latestExport.errorMessage && (
                <p className="text-destructive">
                  {latestExport.errorMessage}
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
                    Copy path
                  </Button>
                  {copyStatus === "copied" && (
                    <span className="text-emerald-700 dark:text-emerald-300">
                      Copied
                    </span>
                  )}
                  {copyStatus === "failed" && (
                    <span className="text-destructive">
                      Copy failed
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

function formatTimestamp(timestamp: string) {
  const date = new Date(timestamp);

  if (Number.isNaN(date.getTime())) {
    return timestamp;
  }

  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(date);
}
