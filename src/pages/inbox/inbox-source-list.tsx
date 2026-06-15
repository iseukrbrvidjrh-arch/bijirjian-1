import { useState } from "react";
import { Link } from "react-router-dom";

import { Button } from "@/components/ui/button";
import { StatusBadge } from "@/components/ui/status-badge";
import {
  formatFileSize,
  parsePdfSourceMetadata,
} from "@/features/capture/pdf-source-metadata";
import {
  useInboxSources,
  useMarkSourceDismissed,
  useMarkSourceProcessed,
} from "@/features/capture/source-queries";
import { InboxSearchBar } from "@/features/inbox/inbox-search-bar";
import { useCreateKnowledgeDraftFromLatestSummary } from "@/features/knowledge/knowledge-queries";
import {
  useLatestSourceSummary,
  useSummarizeSource,
} from "@/features/summary/source-summary-queries";
import {
  aiRunStatusLabel,
  formatDateTime,
  formatUiError,
  inboxStatusLabel,
  providerModelLabel,
  providerTypeLabel,
  sourceTypeLabel,
} from "@/lib/display";
import type { SourceDto } from "@/types/source";

export function InboxSourceList() {
  const [appliedQuery, setAppliedQuery] = useState<string>();
  const inboxQuery = useInboxSources({ query: appliedQuery });
  const isRefreshing = inboxQuery.isFetching && !inboxQuery.isPending;

  function updateQuery(query?: string) {
    const normalizedQuery = query?.trim();
    setAppliedQuery(normalizedQuery || undefined);
  }

  return (
    <div className="space-y-4">
      <InboxSearchBar
        query={appliedQuery}
        isRefreshing={isRefreshing}
        onQueryChange={updateQuery}
        onRefresh={() => void inboxQuery.refetch()}
      />

      {inboxQuery.isPending && (
        <StatusMessage>正在加载收集箱…</StatusMessage>
      )}

      {inboxQuery.isError && (
        <StatusMessage tone="error">
          <span>
            收集箱加载失败：
            {formatUiError(inboxQuery.error)}
          </span>
          <Button
            size="sm"
            type="button"
            variant="outline"
            onClick={() => void inboxQuery.refetch()}
          >
            重试
          </Button>
        </StatusMessage>
      )}

      {inboxQuery.isSuccess && inboxQuery.data.length === 0 && (
        <StatusMessage>
          <span>
            {appliedQuery
              ? `没有找到包含“${appliedQuery}”的内容。`
              : "收集箱还是空的，可以先添加一条文字或导入 PDF。"}
          </span>
          {appliedQuery && (
            <Button
              size="sm"
              type="button"
              variant="outline"
              onClick={() => updateQuery(undefined)}
            >
              清空搜索
            </Button>
          )}
        </StatusMessage>
      )}

      {inboxQuery.isSuccess && inboxQuery.data.length > 0 && (
        <div className="space-y-3">
          <div className="flex flex-wrap items-center justify-between gap-2">
            <h2 className="text-sm font-medium text-muted-foreground">
              待处理内容
            </h2>
            <span className="text-xs text-muted-foreground">
              共 {inboxQuery.data.length} 条
            </span>
          </div>
          <ul className="space-y-3">
            {inboxQuery.data.map((source) => (
              <InboxSourceItem key={source.id} source={source} />
            ))}
          </ul>
        </div>
      )}
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
  const pdfMetadata =
    source.sourceType === "pdf"
      ? parsePdfSourceMetadata(source.metadataJson)
      : null;

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
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-2">
            <StatusBadge tone="blue">
              {sourceTypeLabel(source.sourceType)}
            </StatusBadge>
            <StatusBadge tone="blue">
              {inboxStatusLabel(source.inboxStatus)}
            </StatusBadge>
            {pdfMetadata && (
              <span className="break-all text-sm font-medium">
                {pdfMetadata.originalFileName}
              </span>
            )}
          </div>
          {pdfMetadata && (
            <p className="mt-1 text-xs text-muted-foreground">
              {formatFileSize(pdfMetadata.fileSize)} ·{" "}
              提取 {pdfMetadata.extractedTextLength.toLocaleString("zh-CN")} 字
            </p>
          )}
        </div>
      </div>

      <p className="mt-3 whitespace-pre-wrap break-words text-sm">
        {source.sourceType === "pdf"
          ? textPreview(source.rawContent, 600)
          : source.rawContent}
      </p>

      <div className="mt-3 flex flex-wrap items-center justify-between gap-3">
        <time
          className="text-xs text-muted-foreground"
          dateTime={source.capturedAt}
        >
          {formatDateTime(source.capturedAt)}
        </time>

        <div className="flex flex-wrap items-center gap-2">
          <Button asChild size="sm" variant="ghost">
            <Link to={`/sources/${source.id}`}>查看详情</Link>
          </Button>
          <Button
            size="sm"
            type="button"
            variant="outline"
            disabled={isPending}
            onClick={() => void summarize()}
          >
            {summaryMutation.isPending ? "正在生成总结…" : "生成总结"}
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
              ? "正在创建草稿…"
              : "创建知识草稿"}
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
            标记为已处理
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
            忽略
          </Button>
        </div>
      </div>

      {latestSummary?.status === "succeeded" && latestSummary.summary && (
        <section className="mt-4 rounded-lg border border-violet-200 bg-violet-50/70 p-4">
          <div className="flex flex-wrap items-center gap-2">
            <h3 className="text-sm font-semibold">AI 总结</h3>
            <StatusBadge tone="green">
              {aiRunStatusLabel(latestSummary.status)}
            </StatusBadge>
          </div>
          <p className="mt-2 whitespace-pre-wrap break-words text-sm">
            {latestSummary.summary}
          </p>
          <p className="mt-3 text-xs text-muted-foreground">
            {latestSummary.providerType
              ? providerTypeLabel(latestSummary.providerType)
              : "AI 服务"}
            {" · "}
            {latestSummary.model
              ? providerModelLabel(latestSummary.model)
              : "模型未知"}
            {" · "}提示词版本 {latestSummary.promptVersion}
          </p>
        </section>
      )}

      {latestSummary?.status === "failed" && (
        <section className="mt-4 rounded-md border border-destructive/30 bg-destructive/5 p-4">
          <h3 className="text-sm font-semibold text-destructive">
            最近一次总结失败
          </h3>
          <p className="mt-2 text-sm text-destructive">
            {formatUiError(
              latestSummary.errorMessage,
              "AI 总结失败，请稍后重试。",
            )}
          </p>
        </section>
      )}

      <div className="mt-2 min-h-5 text-xs" aria-live="polite">
        {latestSummaryQuery.isPending && (
          <span className="text-muted-foreground">
            正在加载最近总结…
          </span>
        )}
        {latestSummaryQuery.isError && (
          <span className="text-destructive" role="alert">
            最近总结加载失败：
            {formatUiError(latestSummaryQuery.error)}
          </span>
        )}
        {summaryMutation.isPending && (
          <span className="text-muted-foreground">正在生成总结…</span>
        )}
        {draftMutation.isPending && (
          <span className="text-muted-foreground">
            正在创建知识草稿…
          </span>
        )}
        {draftMutation.isSuccess && (
          <span className="text-emerald-700">知识草稿已创建。</span>
        )}
        {processedMutation.isPending && (
          <span className="text-muted-foreground">
            正在标记为已处理…
          </span>
        )}
        {dismissedMutation.isPending && (
          <span className="text-muted-foreground">正在忽略…</span>
        )}
        {mutationError && (
          <span className="text-destructive" role="alert">
            {formatUiError(mutationError)}
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
    <div
      className={
        tone === "error"
          ? "flex flex-wrap items-center justify-between gap-3 rounded-lg border border-destructive/30 bg-destructive/5 p-4 text-sm text-destructive"
          : "flex flex-wrap items-center justify-between gap-3 rounded-lg border bg-background p-4 text-sm text-muted-foreground"
      }
      role={tone === "error" ? "alert" : "status"}
    >
      {children}
    </div>
  );
}

function textPreview(content: string, limit: number) {
  const characters = Array.from(content);

  if (characters.length <= limit) {
    return content;
  }

  return `${characters.slice(0, limit).join("")}…`;
}
