import { useState } from "react";

import { Button } from "@/components/ui/button";
import {
  useInboxSources,
  useMarkSourceDismissed,
  useMarkSourceProcessed,
} from "@/features/capture/source-queries";
import { useSummarizeSource } from "@/features/summary/source-summary-queries";
import type { SourceDto } from "@/types/source";
import type { SourceSummaryDto } from "@/types/summary";

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
  const [summary, setSummary] = useState<SourceSummaryDto>();
  const isPending =
    processedMutation.isPending ||
    dismissedMutation.isPending ||
    summaryMutation.isPending;
  const mutationError =
    processedMutation.error ??
    dismissedMutation.error ??
    summaryMutation.error;

  async function summarize() {
    processedMutation.reset();
    dismissedMutation.reset();
    summaryMutation.reset();

    try {
      const result = await summaryMutation.mutateAsync(source.id);
      setSummary(result);
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
            disabled={isPending}
            onClick={() => {
              dismissedMutation.reset();
              summaryMutation.reset();
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
              dismissedMutation.mutate(source.id);
            }}
          >
            Dismiss
          </Button>
        </div>
      </div>

      {summary && (
        <section className="mt-4 rounded-md border bg-muted/30 p-4">
          <h3 className="text-sm font-semibold">AI Summary</h3>
          <p className="mt-2 whitespace-pre-wrap break-words text-sm">
            {summary.summary}
          </p>
          <p className="mt-3 text-xs text-muted-foreground">
            {summary.providerType} · {summary.model} · Prompt version{" "}
            {summary.promptVersion}
          </p>
        </section>
      )}

      <div className="mt-2 min-h-5 text-xs" aria-live="polite">
        {summaryMutation.isPending && (
          <span className="text-muted-foreground">Summarizing...</span>
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
