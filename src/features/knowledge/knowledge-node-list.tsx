import { useState } from "react";

import { Button } from "@/components/ui/button";
import { useExportKnowledgeNode } from "@/features/knowledge/knowledge-export-queries";
import {
  useAcceptKnowledgeNode,
  useArchiveKnowledgeNode,
} from "@/features/knowledge/knowledge-queries";
import type { KnowledgeNodeDto } from "@/types/knowledge";

interface KnowledgeNodeListProps {
  nodes: KnowledgeNodeDto[];
  isPending: boolean;
  error: Error | null;
  hasActiveFilters: boolean;
  onRetry: () => void;
  onClearFilters: () => void;
}

export function KnowledgeNodeList({
  nodes,
  isPending,
  error,
  hasActiveFilters,
  onRetry,
  onClearFilters,
}: KnowledgeNodeListProps) {
  if (isPending) {
    return <KnowledgeState>Loading knowledge...</KnowledgeState>;
  }

  if (error && nodes.length === 0) {
    return (
      <KnowledgeState tone="error">
        <span>Could not load knowledge: {error.message}</span>
        <Button
          className="mt-3"
          size="sm"
          type="button"
          variant="outline"
          onClick={onRetry}
        >
          Retry
        </Button>
      </KnowledgeState>
    );
  }

  if (nodes.length === 0) {
    return (
      <KnowledgeState>
        <span>
          {hasActiveFilters
            ? "No knowledge matches the current filters."
            : "No knowledge nodes yet. Create the first one above."}
        </span>
        {hasActiveFilters && (
          <Button
            className="mt-3"
            size="sm"
            type="button"
            variant="outline"
            onClick={onClearFilters}
          >
            Clear filters
          </Button>
        )}
      </KnowledgeState>
    );
  }

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between gap-3">
        <h2 className="text-sm font-medium text-muted-foreground">
          Knowledge nodes
        </h2>
        {error && (
          <span className="text-xs text-destructive" role="alert">
            Refresh failed: {error.message}
          </span>
        )}
      </div>
      <ul className="space-y-3">
        {nodes.map((node) => (
          <KnowledgeNodeItem key={node.id} node={node} />
        ))}
      </ul>
    </div>
  );
}

function KnowledgeNodeItem({ node }: { node: KnowledgeNodeDto }) {
  const [isExpanded, setIsExpanded] = useState(false);
  const acceptMutation = useAcceptKnowledgeNode();
  const archiveMutation = useArchiveKnowledgeNode();
  const exportMutation = useExportKnowledgeNode();
  const isPending =
    acceptMutation.isPending ||
    archiveMutation.isPending ||
    exportMutation.isPending;
  const mutationError =
    acceptMutation.error ?? archiveMutation.error ?? exportMutation.error;
  const hasLongContent =
    node.content.length > 360 || node.content.split("\n").length > 6;

  function acceptNode() {
    archiveMutation.reset();
    exportMutation.reset();
    acceptMutation.mutate(node.id);
  }

  function archiveNode() {
    acceptMutation.reset();
    exportMutation.reset();
    archiveMutation.mutate(node.id);
  }

  function exportNode() {
    acceptMutation.reset();
    archiveMutation.reset();
    exportMutation.mutate(node.id);
  }

  return (
    <li className="rounded-lg border bg-background p-4">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0 flex-1">
          <h3 className="font-semibold">{node.title}</h3>
          <div className="mt-2 flex flex-wrap items-center gap-2 text-xs">
            <KnowledgeStatusLabel status={node.status} />
            <KnowledgeTypeLabel knowledgeType={node.knowledgeType} />
          </div>
        </div>
        <dl className="shrink-0 space-y-1 text-right text-xs text-muted-foreground">
          <div>
            <dt className="inline font-medium">Created </dt>
            <dd className="inline">
              <time dateTime={node.createdAt}>
                {formatTimestamp(node.createdAt)}
              </time>
            </dd>
          </div>
          <div>
            <dt className="inline font-medium">Updated </dt>
            <dd className="inline">
              <time dateTime={node.updatedAt}>
                {formatTimestamp(node.updatedAt)}
              </time>
            </dd>
          </div>
        </dl>
      </div>

      <div className="mt-4">
        <p
          className={`whitespace-pre-wrap break-words text-sm leading-6 ${
            hasLongContent && !isExpanded
              ? "max-h-32 overflow-hidden"
              : ""
          }`}
        >
          {node.content}
        </p>
        {hasLongContent && (
          <Button
            className="mt-2 px-0"
            size="sm"
            type="button"
            variant="link"
            aria-expanded={isExpanded}
            onClick={() => setIsExpanded((current) => !current)}
          >
            {isExpanded ? "Show less" : "Show more"}
          </Button>
        )}
      </div>

      {node.status === "proposed" && (
        <div className="mt-4">
          <div className="flex flex-wrap items-center gap-2">
            <Button
              size="sm"
              type="button"
              disabled={isPending}
              onClick={acceptNode}
            >
              {acceptMutation.isPending ? "Accepting…" : "Accept"}
            </Button>
            <Button
              size="sm"
              type="button"
              variant="outline"
              disabled={isPending}
              onClick={archiveNode}
            >
              {archiveMutation.isPending ? "Archiving…" : "Archive"}
            </Button>
          </div>

          <div className="mt-2 min-h-5 text-xs" aria-live="polite">
            {mutationError && (
              <span className="text-destructive" role="alert">
                {mutationError.message}
              </span>
            )}
          </div>
        </div>
      )}

      {node.status === "accepted" && (
        <div className="mt-4">
          <Button
            size="sm"
            type="button"
            variant="outline"
            disabled={isPending}
            onClick={exportNode}
          >
            {exportMutation.isPending ? "Exporting…" : "Export"}
          </Button>

          <div className="mt-2 min-h-5 text-xs" aria-live="polite">
            {mutationError && (
              <span className="text-destructive" role="alert">
                {mutationError.message}
              </span>
            )}
            {!mutationError &&
              exportMutation.isSuccess &&
              exportMutation.data.exportPath && (
                <span className="break-all text-emerald-700 dark:text-emerald-300">
                  Exported to {exportMutation.data.exportPath}
                </span>
              )}
          </div>
        </div>
      )}
    </li>
  );
}

function KnowledgeTypeLabel({
  knowledgeType,
}: {
  knowledgeType: KnowledgeNodeDto["knowledgeType"];
}) {
  return (
    <span className="rounded-full border border-sky-500/30 bg-sky-500/10 px-2 py-0.5 font-medium capitalize text-sky-700 dark:text-sky-300">
      {knowledgeType}
    </span>
  );
}

function KnowledgeStatusLabel({
  status,
}: {
  status: KnowledgeNodeDto["status"];
}) {
  const tone =
    status === "proposed"
      ? "border-amber-500/30 bg-amber-500/10 text-amber-700 dark:text-amber-300"
      : status === "accepted"
        ? "border-emerald-500/30 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300"
        : "border-border bg-muted text-muted-foreground";

  return (
    <span
      className={`rounded-full border px-2 py-0.5 font-medium capitalize ${tone}`}
    >
      {status}
    </span>
  );
}

function KnowledgeState({
  children,
  tone = "muted",
}: {
  children: React.ReactNode;
  tone?: "muted" | "error";
}) {
  return (
    <div
      className={
        tone === "error"
          ? "rounded-lg border border-destructive/30 bg-destructive/5 p-4 text-sm text-destructive"
          : "rounded-lg border bg-background p-4 text-sm text-muted-foreground"
      }
      role={tone === "error" ? "alert" : "status"}
    >
      {children}
    </div>
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
