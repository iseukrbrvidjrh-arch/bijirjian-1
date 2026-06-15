import { useState } from "react";
import { Link, useParams } from "react-router-dom";

import { Button } from "@/components/ui/button";
import { StatusBadge } from "@/components/ui/status-badge";
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
import {
  aiRunStatusLabel,
  formatDateTime,
  formatUiError,
  inboxStatusLabel,
  knowledgeStatusLabel,
  knowledgeTypeLabel,
  providerModelLabel,
  providerTypeLabel,
  sourceTypeLabel,
} from "@/lib/display";
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
        <span>缺少内容 ID，无法打开详情。</span>
        <Button asChild size="sm" variant="outline">
          <Link to="/inbox">返回收集箱</Link>
        </Button>
      </PageState>
    );
  }

  if (detailQuery.isPending) {
    return <PageState>正在加载内容详情…</PageState>;
  }

  if (detailQuery.isError || !detail || !source) {
    return (
      <PageState tone="error">
        <span>
          内容详情加载失败：
          {detailQuery.error
            ? formatUiError(detailQuery.error)
            : "没有找到这条内容。"}
        </span>
        <div className="flex gap-2">
          <Button
            size="sm"
            type="button"
            variant="outline"
            onClick={() => void detailQuery.refetch()}
          >
            重试
          </Button>
          <Button asChild size="sm" variant="outline">
            <Link to="/inbox">返回收集箱</Link>
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
            <Link to="/inbox">返回收集箱</Link>
          </Button>
          <div className="mt-3 flex flex-wrap items-center gap-2">
            <h1 className="text-2xl font-semibold">内容详情</h1>
            <StatusBadge tone="blue">
              {sourceTypeLabel(source.sourceType)}
            </StatusBadge>
            <StatusBadge
              tone={
                source.inboxStatus === "processed"
                  ? "green"
                  : source.inboxStatus === "unprocessed"
                    ? "blue"
                    : source.inboxStatus === "failed"
                      ? "red"
                      : "gray"
              }
            >
              {inboxStatusLabel(source.inboxStatus)}
            </StatusBadge>
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
          {detailQuery.isFetching ? "正在刷新…" : "刷新"}
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
        <h2 className="text-sm font-semibold">原始内容</h2>
        <div className="mt-3 max-h-[32rem] overflow-auto rounded-md border bg-muted/20 p-4">
          <p className="whitespace-pre-wrap break-words text-sm">
            {source.rawContent}
          </p>
        </div>
      </section>

      <SummarySection summary={detail.latestSummary} />

      <section className="rounded-lg border bg-background p-4">
        <h2 className="text-sm font-semibold">关联知识</h2>
        {detail.relatedKnowledge ? (
          <div className="mt-3 rounded-md border p-3">
            <p className="font-medium">{detail.relatedKnowledge.title}</p>
            <div className="mt-2 flex flex-wrap gap-2">
              <StatusBadge tone="violet">
                {knowledgeTypeLabel(
                  detail.relatedKnowledge.knowledgeType,
                )}
              </StatusBadge>
              <StatusBadge
                tone={
                  detail.relatedKnowledge.status === "accepted"
                    ? "green"
                    : detail.relatedKnowledge.status === "proposed"
                      ? "amber"
                      : "gray"
                }
              >
                {knowledgeStatusLabel(detail.relatedKnowledge.status)}
              </StatusBadge>
            </div>
          </div>
        ) : (
          <p className="mt-3 text-sm text-muted-foreground">
            这条内容还没有关联知识。
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
      <h2 className="text-sm font-semibold">操作</h2>
      <div className="mt-3 flex flex-wrap gap-2">
        <Button
          type="button"
          variant="outline"
          disabled={isPending}
          onClick={() => void onAction("summary")}
        >
          {summaryPending ? "正在生成总结…" : "生成总结"}
        </Button>
        <Button
          type="button"
          variant="outline"
          disabled={isPending || !canCreateDraft}
          onClick={() => void onAction("draft")}
        >
          {draftPending ? "正在创建草稿…" : "创建知识草稿"}
        </Button>
        <Button
          type="button"
          disabled={isPending || !canTransition}
          onClick={() => void onAction("processed")}
        >
          {processedPending ? "正在标记…" : "标记为已处理"}
        </Button>
        <Button
          type="button"
          variant="outline"
          disabled={isPending || !canTransition}
          onClick={() => void onAction("dismissed")}
        >
          {dismissedPending ? "正在忽略…" : "忽略"}
        </Button>
      </div>
      <div className="mt-3 min-h-5 text-sm" aria-live="polite">
        {hasRelatedKnowledge && (
          <p className="text-muted-foreground">
            这条内容已经关联知识草稿，不能重复创建。
          </p>
        )}
        {!canTransition && (
          <p className="text-muted-foreground">
            这条内容当前为“{inboxStatusLabel(source.inboxStatus)}”状态。
          </p>
        )}
        {error && (
          <p className="text-destructive" role="alert">
            {formatUiError(error)}
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
      <h2 className="text-sm font-semibold">内容信息</h2>
      <dl className="mt-3 grid gap-3 text-sm sm:grid-cols-2">
        <MetadataItem
          label="收集时间"
          value={formatDateTime(source.capturedAt)}
        />
        <MetadataItem
          label="处理时间"
          value={
            source.processedAt
              ? formatDateTime(source.processedAt)
              : "尚未处理"
          }
        />
        <MetadataItem
          label="创建时间"
          value={formatDateTime(source.createdAt)}
        />
        <MetadataItem
          label="更新时间"
          value={formatDateTime(source.updatedAt)}
        />
        {pdfMetadata && (
          <>
            <MetadataItem
              label="原始文件"
              value={pdfMetadata.originalFileName}
            />
            <MetadataItem
              label="文件大小"
              value={formatFileSize(pdfMetadata.fileSize)}
            />
            <MetadataItem
              label="提取文字"
              value={`${pdfMetadata.extractedTextLength.toLocaleString("zh-CN")} 字`}
            />
            <MetadataItem
              label="添加方式"
              value="PDF 导入"
            />
          </>
        )}
      </dl>
      {!pdfMetadata && (
        <p className="mt-3 text-xs text-muted-foreground">
          这条文本内容没有额外文件信息。
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
      <h2 className="text-sm font-semibold">最近一次 AI 运行记录</h2>
      {!summary && (
        <p className="mt-3 text-sm text-muted-foreground">
          这条内容还没有生成总结。
        </p>
      )}
      {summary && (
        <div className="mt-3 space-y-3">
          <div className="flex flex-wrap gap-2">
            <StatusBadge
              tone={summary.status === "succeeded" ? "green" : "red"}
            >
              {aiRunStatusLabel(summary.status)}
            </StatusBadge>
            {summary.providerType && (
              <StatusBadge tone="violet">
                {providerTypeLabel(summary.providerType)}
              </StatusBadge>
            )}
            {summary.model && (
              <StatusBadge tone="blue">
                {providerModelLabel(summary.model)}
              </StatusBadge>
            )}
          </div>
          {summary.summary && (
            <p className="whitespace-pre-wrap break-words text-sm">
              {summary.summary}
            </p>
          )}
          {summary.errorMessage && (
            <p className="text-sm text-destructive">
              {formatUiError(summary.errorMessage)}
            </p>
          )}
          <dl className="grid gap-3 text-sm sm:grid-cols-2">
            <MetadataItem
              label="提示词版本"
              value={
                summary.promptVersion === null
                  ? "不可用"
                  : String(summary.promptVersion)
              }
            />
            <MetadataItem
              label="创建时间"
              value={formatDateTime(summary.createdAt)}
            />
            <MetadataItem
              label="完成时间"
              value={formatDateTime(summary.completedAt)}
            />
          </dl>
        </div>
      )}
    </section>
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
      return "总结已更新。";
    case "draft":
      return "知识草稿已创建。";
    case "processed":
      return "已标记为已处理。";
    case "dismissed":
      return "已忽略这条内容。";
    default:
      return undefined;
  }
}
