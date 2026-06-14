import { useInboxSources } from "@/features/capture/source-queries";
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
  return (
    <li className="rounded-lg border bg-background p-4">
      <p className="whitespace-pre-wrap break-words text-sm">
        {source.rawContent}
      </p>
      <time
        className="mt-3 block text-xs text-muted-foreground"
        dateTime={source.capturedAt}
      >
        {formatCapturedAt(source.capturedAt)}
      </time>
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
