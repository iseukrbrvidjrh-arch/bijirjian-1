import { useState } from "react";
import { Link, useParams } from "react-router-dom";

import { Button } from "@/components/ui/button";
import {
  formatFileSize,
  parsePdfSourceMetadata,
} from "@/features/capture/pdf-source-metadata";
import {
  useMarkSourceDismissed,
  useMarkSourceProcessed,
  useSourceDetail,
} from "@/features/capture/source-queries";
import { useCreateKnowledgeDraftFromLatestSummary } from "@/features/knowledge/knowledge-queries";
import { useSummarizeSource } from "@/features/summary/source-summary-queries";
import type { SourceDto } from "@/types/source";
import type { LatestSourceSummaryDto } from "@/types/summary";

export function SourceDetailPage() {
  const { sourceId = "" } = useParams();
  const detailQuery = useSourceDetail(sourceId, sourceId.length > 0);
  const processedMutation = useMarkSourceProcessed();
  const dismissedMutation = useMarkSourceDismissed();
  const summaryMutation = useSummarizeSource();
  const draftMutation = useCreateKnowledgeDraftFromLatestSummary();
  const [lastAction, setLastAction] = useState<
    "summary" | "draft" | "processed" | "dismissed"
  >();
  const detail = detailQuery.data;
  const source = detail?.source;
  const isActionPending =
    processedMutation.isPending ||
    dismissedMutation.isPending ||
    summaryMutation.isPending ||
    draftMutation.isPending;
  const actionError =
    processedMutation.error ??
    dismissedMutation.error ??
    summaryMutation.error ??
    draftMutation.error;

  function resetActions() {
    processedMutation.reset();
    dismissedMutation.reset();
    summaryMutation.reset();
    draftMutation.reset();
    setLastAction(undefined);
  }

  async function runAction(
    action: "summary" | "draft" | "processed" | "dismissed",
  ) {
    resetActions();

    try {
      if (action === "summary") {
        await summaryMutation.mutateAsync(sourceId);
      } else if (action === "draft") {
        await draftMutation.mutateAsync(sourceId);
      } else if (action === "processed") {
        await processedMutation.mutateAsync(sourceId);
      } else {
        await dismissedMutation.mutateAsync(sourceId);
      }
      setLastAction(action);
    } catch {
      // Mutation state renders the action error.
    }
  }

  if (!sourceId) {
    return (
      <PageState tone="error">
        <span>The Source ID is missing.</span>
        <Button asChild size="sm" variant="outline">
          <Link to="/inbox">Back to Inbox</Link>
        </Button>
      </PageState>
    );
  }

  if (detailQuery.isPending) {
    return <PageState>Loading Source details...</PageState>;
  }

  if (detailQuery.isError || !detail || !source) {
    return (
      <PageState tone="error">
        <span>
          Could not load Source details:{" "}
          {detailQuery.error?.message ?? "Source not found."}
        </span>
        <div className="flex gap-2">
          <Button
            size="sm"
            type="button"
            variant="outline"
            onClick={() => void detailQuery.refetch()}
          >
            Retry
          </Button>
          <Button asChild size="sm" variant="outline">
            <Link to="/inbox">Back to Inbox</Link>
          </Button>
        </div>
      </PageState>
    );
  }

  const canTransition = source.inboxStatus === "unprocessed";
  const canCreateDraft =
    detail.latestSummary?.status === "succeeded" &&
    Boolean(detail.latestSummary.summary) &&
    detail.relatedKnowledge === null;

  return (
    <section className="mx-auto max-w-5xl space-y-6">
      <header className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <Button asChild size="sm" variant="ghost">
            <Link to="/inbox">Back to Inbox</Link>
          </Button>
          <div className="mt-3 flex flex-wrap items-center gap-2">
            <h1 className="text-2xl font-semibold">Source Details</h1>
            <Badge>{source.sourceType}</Badge>
            <Badge>{source.inboxStatus}</Badge>
          </div>
          <p className="mt-2 break-all text-xs text-muted-foreground">
            {source.id}
          </p>
        </div>
        <Button
          type="button"
          variant="outline"
          disabled={detailQuery.isFetching}
          onClick={() => void detailQuery.refetch()}
        >
          {detailQuery.isFetching ? "Refreshing..." : "Refresh"}
        </Button>
      </header>

      <SourceActions
        source={source}
        hasRelatedKnowledge={detail.relatedKnowledge !== null}
        canCreateDraft={canCreateDraft}
        isPending={isActionPending}
        summaryPending={summaryMutation.isPending}
        draftPending={draftMutation.isPending}
        processedPending={processedMutation.isPending}
        dismissedPending={dismissedMutation.isPending}
        canTransition={canTransition}
        error={actionError?.message}
        successMessage={actionSuccessMessage(lastAction)}
        onAction={runAction}
      />

      <SourceMetadata source={source} />

      <section className="rounded-lg border bg-background p-4">
        <h2 className="text-sm font-semibold">Source Content</h2>
        <div className="mt-3 max-h-[32rem] overflow-auto rounded-md border bg-muted/20 p-4">
          <p className="whitespace-pre-wrap break-words text-sm">
            {source.rawContent}
          </p>
        </div>
      </section>

      <SummarySection summary={detail.latestSummary} />

      <section className="rounded-lg border bg-background p-4">
        <h2 className="text-sm font-semibold">Related Knowledge</h2>
        {detail.relatedKnowledge ? (
          <div className="mt-3 rounded-md border p-3">
            <p className="font-medium">{detail.relatedKnowledge.title}</p>
            <div className="mt-2 flex flex-wrap gap-2">
              <Badge>{detail.relatedKnowledge.knowledgeType}</Badge>
              <Badge>{detail.relatedKnowledge.status}</Badge>
            </div>
          </div>
        ) : (
          <p className="mt-3 text-sm text-muted-foreground">
            No related Knowledge yet.
          </p>
        )}
      </section>
    </section>
  );
}

function SourceActions({
  source,
  hasRelatedKnowledge,
  canCreateDraft,
  canTransition,
  isPending,
  summaryPending,
  draftPending,
  processedPending,
  dismissedPending,
  error,
  successMessage,
  onAction,
}: {
  source: SourceDto;
  hasRelatedKnowledge: boolean;
  canCreateDraft: boolean;
  canTransition: boolean;
  isPending: boolean;
  summaryPending: boolean;
  draftPending: boolean;
  processedPending: boolean;
  dismissedPending: boolean;
  error?: string;
  successMessage?: string;
  onAction: (
    action: "summary" | "draft" | "processed" | "dismissed",
  ) => Promise<void>;
}) {
  return (
    <section className="rounded-lg border bg-background p-4">
      <h2 className="text-sm font-semibold">Actions</h2>
      <div className="mt-3 flex flex-wrap gap-2">
        <Button
          type="button"
          variant="outline"
          disabled={isPending}
          onClick={() => void onAction("summary")}
        >
          {summaryPending ? "Summarizing..." : "Summarize"}
        </Button>
        <Button
          type="button"
          variant="outline"
          disabled={isPending || !canCreateDraft}
          onClick={() => void onAction("draft")}
        >
          {draftPending ? "Creating draft..." : "Create Knowledge Draft"}
        </Button>
        <Button
          type="button"
          disabled={isPending || !canTransition}
          onClick={() => void onAction("processed")}
        >
          {processedPending ? "Marking processed..." : "Mark processed"}
        </Button>
        <Button
          type="button"
          variant="outline"
          disabled={isPending || !canTransition}
          onClick={() => void onAction("dismissed")}
        >
          {dismissedPending ? "Dismissing..." : "Dismiss"}
        </Button>
      </div>
      <div className="mt-3 min-h-5 text-sm" aria-live="polite">
        {hasRelatedKnowledge && (
          <p className="text-muted-foreground">
            This Source already has related Knowledge.
          </p>
        )}
        {!canTransition && (
          <p className="text-muted-foreground">
            Source lifecycle is already {source.inboxStatus}.
          </p>
        )}
        {error && (
          <p className="text-destructive" role="alert">
            {error}
          </p>
        )}
        {successMessage && <p>{successMessage}</p>}
      </div>
    </section>
  );
}

function SourceMetadata({ source }: { source: SourceDto }) {
  const pdfMetadata =
    source.sourceType === "pdf"
      ? parsePdfSourceMetadata(source.metadataJson)
      : null;

  return (
    <section className="rounded-lg border bg-background p-4">
      <h2 className="text-sm font-semibold">Metadata</h2>
      <dl className="mt-3 grid gap-3 text-sm sm:grid-cols-2">
        <MetadataItem label="Captured" value={formatDate(source.capturedAt)} />
        <MetadataItem
          label="Processed"
          value={
            source.processedAt ? formatDate(source.processedAt) : "Not processed"
          }
        />
        <MetadataItem label="Created" value={formatDate(source.createdAt)} />
        <MetadataItem label="Updated" value={formatDate(source.updatedAt)} />
        {pdfMetadata && (
          <>
            <MetadataItem
              label="Original file"
              value={pdfMetadata.originalFileName}
            />
            <MetadataItem
              label="File size"
              value={formatFileSize(pdfMetadata.fileSize)}
            />
            <MetadataItem
              label="Extracted text"
              value={`${pdfMetadata.extractedTextLength.toLocaleString()} characters`}
            />
            <MetadataItem
              label="Captured via"
              value={pdfMetadata.capturedVia}
            />
          </>
        )}
      </dl>
      {!pdfMetadata && (
        <p className="mt-3 text-xs text-muted-foreground">
          No additional structured metadata.
        </p>
      )}
    </section>
  );
}

function MetadataItem({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <dt className="text-xs font-medium text-muted-foreground">{label}</dt>
      <dd className="mt-1 break-words">{value}</dd>
    </div>
  );
}

function SummarySection({
  summary,
}: {
  summary: LatestSourceSummaryDto | null;
}) {
  return (
    <section className="rounded-lg border bg-background p-4">
      <h2 className="text-sm font-semibold">Latest AI Run</h2>
      {!summary && (
        <p className="mt-3 text-sm text-muted-foreground">
          This Source has not been summarized.
        </p>
      )}
      {summary && (
        <div className="mt-3 space-y-3">
          <div className="flex flex-wrap gap-2">
            <Badge>{summary.status}</Badge>
            {summary.providerType && <Badge>{summary.providerType}</Badge>}
            {summary.model && <Badge>{summary.model}</Badge>}
          </div>
          {summary.summary && (
            <p className="whitespace-pre-wrap break-words text-sm">
              {summary.summary}
            </p>
          )}
          {summary.errorMessage && (
            <p className="text-sm text-destructive">
              {summary.errorMessage}
            </p>
          )}
          <dl className="grid gap-3 text-sm sm:grid-cols-2">
            <MetadataItem
              label="Prompt version"
              value={
                summary.promptVersion === null
                  ? "Unavailable"
                  : String(summary.promptVersion)
              }
            />
            <MetadataItem
              label="Created"
              value={formatDate(summary.createdAt)}
            />
            <MetadataItem
              label="Completed"
              value={formatDate(summary.completedAt)}
            />
          </dl>
        </div>
      )}
    </section>
  );
}

function Badge({ children }: { children: React.ReactNode }) {
  return (
    <span className="rounded-full border px-2 py-0.5 text-xs font-medium capitalize">
      {children}
    </span>
  );
}

function PageState({
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
          ? "mx-auto flex max-w-3xl flex-wrap items-center justify-between gap-3 rounded-lg border border-destructive/30 bg-destructive/5 p-4 text-sm text-destructive"
          : "mx-auto max-w-3xl rounded-lg border bg-background p-4 text-sm text-muted-foreground"
      }
      role={tone === "error" ? "alert" : "status"}
    >
      {children}
    </div>
  );
}

function actionSuccessMessage(
  action?: "summary" | "draft" | "processed" | "dismissed",
) {
  switch (action) {
    case "summary":
      return "Summary refreshed.";
    case "draft":
      return "Knowledge draft created.";
    case "processed":
      return "Source marked as processed.";
    case "dismissed":
      return "Source dismissed.";
    default:
      return undefined;
  }
}

function formatDate(value: string) {
  const date = new Date(value);

  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(date);
}
