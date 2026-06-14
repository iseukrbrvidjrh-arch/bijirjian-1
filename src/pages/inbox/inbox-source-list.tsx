import { Button } from "@/components/ui/button";
import {
  useInboxSources,
  useMarkSourceDismissed,
  useMarkSourceProcessed,
} from "@/features/capture/source-queries";
import { useCreateKnowledgeDraftFromLatestSummary } from "@/features/knowledge/knowledge-queries";
import {
  useLatestSourceSummary,
  useSummarizeSource,
} from "@/features/summary/source-summary-queries";
import type { SourceDto } from "@/types/source";

export function InboxSourceList() {
  const inboxQuery = useInboxSources();

  if (inboxQuery.isPending) {
    return <StatusMessage>Loading inbox...</StatusMessage>;
  }

  if (inboxQuery.isError) {
    return (
      <StatusMessage tone="error">
        Could not load the inbox: {inboxQuery.error.message}
      </StatusMessage>
    );
  }

  if (inboxQuery.data.length === 0) {
    return <StatusMessage>Your inbox is empty.</StatusMessage>;
  }

  return (
    <div className="space-y-3">
      <h2 className="text-sm font-medium text-muted-foreground">
        Unprocessed sources
      </h2>
      <ul className="space-y-3">
        {inboxQuery.data.map((source) => (
          <InboxSourceItem key={source.id} source={source} />
        ))}
      </ul>
    </div>
  );
}

function InboxSourceItem({ source }: { source: SourceDto }) {
  const processedMutation = useMarkSourceProcessed();
  const dismissedMutation = useMarkSourceDismissed();
  const summaryMutation = useSummarizeSource();
  const draftMutation = useCreateKnowledgeDraftFromLatestSummary();
  const latestSummaryQuery = useLatestSourceSummary(source.id);
  const latestSummary = latestSummaryQuery.data;
  const isPending =
    processedMutation.isPending ||
    dismissedMutation.isPending ||
    summaryMutation.isPending ||
    draftMutation.isPending;
  const mutationError =
    processedMutation.error ??
    dismissedMutation.error ??
    summaryMutation.error ??
    draftMutation.error;

  async function summarize() {
    processedMutation.reset();
    dismissedMutation.reset();
    summaryMutation.reset();
    draftMutation.reset();

    try {
      await summaryMutation.mutateAsync(source.id);
    } catch {
      // Mutation state renders the error in this source card.
    }
  }

  async function createKnowledgeDraft() {
    processedMutation.reset();
    dismissedMutation.reset();
    summaryMutation.reset();
    draftMutation.reset();

    try {
      await draftMutation.mutateAsync(source.id);
    } catch {
      // Mutation state renders the error in this source card.
    }
  }

  return (
    <li className="rounded-lg border bg-background p-4">
      <p className="whitespace-pre-wrap break-words text-sm">
        {source.rawContent}
      </p>

      <div className="mt-3 flex flex-wrap items-center justify-between gap-3">
        <time
          className="text-xs text-muted-foreground"
          dateTime={source.capturedAt}
        >
          {formatCapturedAt(source.capturedAt)}
        </time>

        <div className="flex flex-wrap items-center gap-2">
          <Button
            size="sm"
            type="button"
            variant="outline"
            disabled={isPending}
            onClick={() => void summarize()}
          >
            {summaryMutation.isPending ? "Summarizing..." : "Summarize"}
          </Button>
          <Button
            size="sm"
            type="button"
            variant="outline"
            disabled={
              isPending ||
              latestSummary?.status !== "succeeded" ||
              !latestSummary.summary ||
              draftMutation.isSuccess
            }
            onClick={() => void createKnowledgeDraft()}
          >
            {draftMutation.isPending
              ? "Creating draft..."
              : "Create Knowledge Draft"}
          </Button>
          <Button
            size="sm"
            type="button"
            disabled={isPending}
            onClick={() => {
              dismissedMutation.reset();
              summaryMutation.reset();
              draftMutation.reset();
              processedMutation.mutate(source.id);
            }}
          >
            Mark processed
          </Button>
          <Button
            size="sm"
            type="button"
            variant="outline"
            disabled={isPending}
            onClick={() => {
              processedMutation.reset();
              summaryMutation.reset();
              draftMutation.reset();
              dismissedMutation.mutate(source.id);
            }}
          >
            Dismiss
          </Button>
        </div>
      </div>

      {latestSummary?.status === "succeeded" && latestSummary.summary && (
        <section className="mt-4 rounded-md border bg-muted/30 p-4">
          <h3 className="text-sm font-semibold">AI Summary</h3>
          <p className="mt-2 whitespace-pre-wrap break-words text-sm">
            {latestSummary.summary}
          </p>
          <p className="mt-3 text-xs text-muted-foreground">
            {latestSummary.providerType} · {latestSummary.model} · Prompt
            version {latestSummary.promptVersion}
          </p>
        </section>
      )}

      {latestSummary?.status === "failed" && (
        <section className="mt-4 rounded-md border border-destructive/30 bg-destructive/5 p-4">
          <h3 className="text-sm font-semibold text-destructive">
            Latest summary failed
          </h3>
          <p className="mt-2 text-sm text-destructive">
            {latestSummary.errorMessage ?? "The AI summary request failed."}
          </p>
        </section>
      )}

      <div className="mt-2 min-h-5 text-xs" aria-live="polite">
        {latestSummaryQuery.isPending && (
          <span className="text-muted-foreground">
            Loading latest summary...
          </span>
        )}
        {latestSummaryQuery.isError && (
          <span className="text-destructive" role="alert">
            Could not load the latest summary:{" "}
            {latestSummaryQuery.error.message}
          </span>
        )}
        {summaryMutation.isPending && (
          <span className="text-muted-foreground">Summarizing...</span>
        )}
        {draftMutation.isPending && (
          <span className="text-muted-foreground">
            Creating draft...
          </span>
        )}
        {draftMutation.isSuccess && (
          <span>Knowledge draft created.</span>
        )}
        {processedMutation.isPending && (
          <span className="text-muted-foreground">
            Marking as processed...
          </span>
        )}
        {dismissedMutation.isPending && (
          <span className="text-muted-foreground">Dismissing...</span>
        )}
        {mutationError && (
          <span className="text-destructive" role="alert">
            {mutationError.message}
          </span>
        )}
      </div>
    </li>
  );
}

function StatusMessage({
  children,
  tone = "muted",
}: {
  children: React.ReactNode;
  tone?: "muted" | "error";
}) {
  return (
    <p
      className={
        tone === "error"
          ? "rounded-lg border border-destructive/30 bg-destructive/5 p-4 text-sm text-destructive"
          : "rounded-lg border bg-background p-4 text-sm text-muted-foreground"
      }
      role={tone === "error" ? "alert" : "status"}
    >
      {children}
    </p>
  );
}

function formatCapturedAt(capturedAt: string) {
  const date = new Date(capturedAt);

  if (Number.isNaN(date.getTime())) {
    return capturedAt;
  }

  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(date);
}
