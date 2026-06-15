import { Link } from "react-router-dom";

import { Button } from "@/components/ui/button";
import { useDashboardSummary } from "@/features/dashboard/dashboard-queries";
import type { KnowledgeNodeDto } from "@/types/knowledge";
import type { SourceDto } from "@/types/source";

export function DashboardPage() {
  const dashboardQuery = useDashboardSummary();
  const summary = dashboardQuery.data;
  const isRefreshing =
    dashboardQuery.isFetching && !dashboardQuery.isPending;

  return (
    <section className="mx-auto max-w-5xl">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <h1 className="text-2xl font-semibold">Dashboard</h1>
          <p className="mt-2 text-sm text-muted-foreground">
            A local overview of your inbox, knowledge, and Obsidian
            configuration.
          </p>
        </div>
        <Button
          type="button"
          variant="outline"
          disabled={dashboardQuery.isFetching}
          onClick={() => void dashboardQuery.refetch()}
        >
          {isRefreshing ? "Refreshing..." : "Refresh"}
        </Button>
      </div>

      <div className="mt-6">
        {dashboardQuery.isPending && (
          <DashboardState>Loading dashboard...</DashboardState>
        )}

        {dashboardQuery.isError && (
          <DashboardState tone="error">
            <span>
              Could not load the dashboard: {dashboardQuery.error.message}
            </span>
            <Button
              size="sm"
              type="button"
              variant="outline"
              onClick={() => void dashboardQuery.refetch()}
            >
              Retry
            </Button>
          </DashboardState>
        )}

        {summary && (
          <div className="space-y-6">
            <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-5">
              <MetricCard
                label="Inbox"
                value={summary.inboxUnprocessedCount}
                detail="Unprocessed"
              />
              <MetricCard
                label="Knowledge"
                value={summary.knowledgeTotalCount}
                detail="Total"
              />
              <MetricCard
                label="Proposed"
                value={summary.proposedKnowledgeCount}
                detail="Needs review"
              />
              <MetricCard
                label="Accepted"
                value={summary.acceptedKnowledgeCount}
                detail="Ready to use"
              />
              <MetricCard
                label="Archived"
                value={summary.archivedKnowledgeCount}
                detail="Stored away"
              />
            </div>

            <section className="rounded-lg border bg-background p-4">
              <h2 className="text-sm font-semibold">
                Obsidian Vault
              </h2>
              <p className="mt-2 text-sm text-muted-foreground">
                {summary.obsidianVaultConfigured
                  ? "Configured for the current workspace."
                  : "Not configured for the current workspace."}
              </p>
            </section>

            <div className="grid gap-6 lg:grid-cols-2">
              <RecentKnowledgeList nodes={summary.recentKnowledge} />
              <RecentInboxList sources={summary.recentInboxSources} />
            </div>

            <section className="rounded-lg border bg-background p-4">
              <h2 className="text-sm font-semibold">Quick access</h2>
              <div className="mt-3 flex flex-wrap gap-2">
                <Button asChild>
                  <Link to="/inbox">Go to Inbox</Link>
                </Button>
                <Button asChild variant="outline">
                  <Link to="/knowledge">Go to Knowledge</Link>
                </Button>
                <Button asChild variant="outline">
                  <Link to="/settings">Go to Settings</Link>
                </Button>
              </div>
            </section>
          </div>
        )}
      </div>
    </section>
  );
}

function MetricCard({
  label,
  value,
  detail,
}: {
  label: string;
  value: number;
  detail: string;
}) {
  return (
    <article className="rounded-lg border bg-background p-4">
      <p className="text-sm font-medium text-muted-foreground">
        {label}
      </p>
      <p className="mt-2 text-3xl font-semibold">{value}</p>
      <p className="mt-1 text-xs text-muted-foreground">{detail}</p>
    </article>
  );
}

function RecentKnowledgeList({
  nodes,
}: {
  nodes: KnowledgeNodeDto[];
}) {
  return (
    <section className="rounded-lg border bg-background p-4">
      <div className="flex items-center justify-between gap-3">
        <h2 className="text-sm font-semibold">Recent Knowledge</h2>
        <Button asChild size="sm" variant="ghost">
          <Link to="/knowledge">View all</Link>
        </Button>
      </div>

      {nodes.length === 0 ? (
        <p className="mt-3 text-sm text-muted-foreground">
          No knowledge nodes yet.
        </p>
      ) : (
        <ul className="mt-3 space-y-3">
          {nodes.map((node) => (
            <li key={node.id} className="rounded-md border p-3">
              <p className="truncate text-sm font-medium">{node.title}</p>
              <div className="mt-2 flex flex-wrap gap-2 text-xs text-muted-foreground">
                <span>{formatLabel(node.status)}</span>
                <span>{formatLabel(node.knowledgeType)}</span>
                <time dateTime={node.createdAt}>
                  {formatDate(node.createdAt)}
                </time>
              </div>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

function RecentInboxList({
  sources,
}: {
  sources: SourceDto[];
}) {
  return (
    <section className="rounded-lg border bg-background p-4">
      <div className="flex items-center justify-between gap-3">
        <h2 className="text-sm font-semibold">Recent Inbox</h2>
        <Button asChild size="sm" variant="ghost">
          <Link to="/inbox">View all</Link>
        </Button>
      </div>

      {sources.length === 0 ? (
        <p className="mt-3 text-sm text-muted-foreground">
          No unprocessed sources.
        </p>
      ) : (
        <ul className="mt-3 space-y-3">
          {sources.map((source) => (
            <li key={source.id} className="rounded-md border p-3">
              <p className="max-h-12 overflow-hidden whitespace-pre-wrap break-words text-sm">
                {source.rawContent}
              </p>
              <time
                className="mt-2 block text-xs text-muted-foreground"
                dateTime={source.capturedAt}
              >
                {formatDate(source.capturedAt)}
              </time>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

function DashboardState({
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
          ? "flex flex-wrap items-center justify-between gap-3 rounded-lg border border-destructive/30 bg-destructive/5 p-4 text-sm text-destructive"
          : "rounded-lg border bg-background p-4 text-sm text-muted-foreground"
      }
      role={tone === "error" ? "alert" : "status"}
    >
      {children}
    </div>
  );
}

function formatLabel(value: string) {
  return value.charAt(0).toUpperCase() + value.slice(1);
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
