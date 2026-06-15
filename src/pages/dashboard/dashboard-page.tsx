import { Link } from "react-router-dom";

import { Button } from "@/components/ui/button";
import { StatusBadge } from "@/components/ui/status-badge";
import { useDashboardSummary } from "@/features/dashboard/dashboard-queries";
import {
  formatDateTime,
  formatUiError,
  knowledgeStatusLabel,
  knowledgeTypeLabel,
  sourceTypeLabel,
} from "@/lib/display";
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
          <h1 className="text-2xl font-semibold">总览</h1>
          <p className="mt-2 text-sm text-muted-foreground">
            快速查看收集箱、知识库和 Obsidian 配置状态。
          </p>
        </div>
        <Button
          type="button"
          variant="outline"
          disabled={dashboardQuery.isFetching}
          onClick={() => void dashboardQuery.refetch()}
        >
          {isRefreshing ? "正在刷新…" : "刷新"}
        </Button>
      </div>

      <div className="mt-6">
        {dashboardQuery.isPending && (
          <DashboardState>正在加载总览…</DashboardState>
        )}

        {dashboardQuery.isError && (
          <DashboardState tone="error">
            <span>
              总览加载失败：{formatUiError(dashboardQuery.error)}
            </span>
            <Button
              size="sm"
              type="button"
              variant="outline"
              onClick={() => void dashboardQuery.refetch()}
            >
              重试
            </Button>
          </DashboardState>
        )}

        {summary && (
          <div className="space-y-6">
            <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-5">
              <MetricCard
                label="收集箱"
                value={summary.inboxUnprocessedCount}
                detail="未处理内容"
              />
              <MetricCard
                label="知识总数"
                value={summary.knowledgeTotalCount}
                detail="全部知识"
              />
              <MetricCard
                label="待审核"
                value={summary.proposedKnowledgeCount}
                detail="等待确认"
              />
              <MetricCard
                label="已收录"
                value={summary.acceptedKnowledgeCount}
                detail="可导出使用"
              />
              <MetricCard
                label="已归档"
                value={summary.archivedKnowledgeCount}
                detail="暂不使用"
              />
            </div>

            <section className="rounded-lg border bg-background p-4">
              <h2 className="text-sm font-semibold">
                Obsidian 仓库
              </h2>
              <p className="mt-2 text-sm text-muted-foreground">
                {summary.obsidianVaultConfigured
                  ? "已为当前工作区配置。"
                  : "当前还没有配置 Obsidian 仓库。"}
              </p>
            </section>

            <div className="grid gap-6 lg:grid-cols-2">
              <RecentKnowledgeList nodes={summary.recentKnowledge} />
              <RecentInboxList sources={summary.recentInboxSources} />
            </div>

            <section className="rounded-lg border bg-background p-4">
              <h2 className="text-sm font-semibold">快捷入口</h2>
              <div className="mt-3 flex flex-wrap gap-2">
                <Button asChild>
                  <Link to="/inbox">前往收集箱</Link>
                </Button>
                <Button asChild variant="outline">
                  <Link to="/knowledge">前往知识库</Link>
                </Button>
                <Button asChild variant="outline">
                  <Link to="/settings">前往设置</Link>
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
        <h2 className="text-sm font-semibold">最近知识</h2>
        <Button asChild size="sm" variant="ghost">
          <Link to="/knowledge">查看全部</Link>
        </Button>
      </div>

      {nodes.length === 0 ? (
        <p className="mt-3 text-sm text-muted-foreground">
          暂无知识内容。
        </p>
      ) : (
        <ul className="mt-3 space-y-3">
          {nodes.map((node) => (
            <li key={node.id} className="rounded-md border p-3">
              <p className="truncate text-sm font-medium">{node.title}</p>
              <div className="mt-2 flex flex-wrap gap-2 text-xs text-muted-foreground">
                <StatusBadge
                  tone={
                    node.status === "accepted"
                      ? "green"
                      : node.status === "proposed"
                        ? "amber"
                        : "gray"
                  }
                >
                  {knowledgeStatusLabel(node.status)}
                </StatusBadge>
                <StatusBadge tone="violet">
                  {knowledgeTypeLabel(node.knowledgeType)}
                </StatusBadge>
                <time dateTime={node.createdAt}>
                  {formatDateTime(node.createdAt)}
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
        <h2 className="text-sm font-semibold">最近收集</h2>
        <Button asChild size="sm" variant="ghost">
          <Link to="/inbox">查看全部</Link>
        </Button>
      </div>

      {sources.length === 0 ? (
        <p className="mt-3 text-sm text-muted-foreground">
          暂无未处理内容。
        </p>
      ) : (
        <ul className="mt-3 space-y-3">
          {sources.map((source) => (
            <li key={source.id} className="rounded-md border p-3">
              <StatusBadge tone="blue">
                {sourceTypeLabel(source.sourceType)}
              </StatusBadge>
              <p className="mt-2 max-h-12 overflow-hidden whitespace-pre-wrap break-words text-sm">
                {source.rawContent}
              </p>
              <time
                className="mt-2 block text-xs text-muted-foreground"
                dateTime={source.capturedAt}
              >
                {formatDateTime(source.capturedAt)}
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
