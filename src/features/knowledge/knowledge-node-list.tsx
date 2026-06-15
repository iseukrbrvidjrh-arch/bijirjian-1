import { useState } from "react";

import { Button } from "@/components/ui/button";
import { StatusBadge } from "@/components/ui/status-badge";
import { KnowledgeExportStatus } from "@/features/knowledge/knowledge-export-status";
import {
  useAcceptKnowledgeNode,
  useArchiveKnowledgeNode,
} from "@/features/knowledge/knowledge-queries";
import {
  formatDateTime,
  formatUiError,
  knowledgeStatusLabel,
  knowledgeTypeLabel,
} from "@/lib/display";
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
    return <KnowledgeState>正在加载知识库…</KnowledgeState>;
  }

  if (error && nodes.length === 0) {
    return (
      <KnowledgeState tone="error">
        <span>知识库加载失败：{formatUiError(error)}</span>
        <Button
          className="mt-3"
          size="sm"
          type="button"
          variant="outline"
          onClick={onRetry}
        >
          重试
        </Button>
      </KnowledgeState>
    );
  }

  if (nodes.length === 0) {
    return (
      <KnowledgeState>
        <span>
          {hasActiveFilters
            ? "当前搜索或筛选条件下没有结果。"
            : "知识库还是空的，可以在上方手动添加一条知识。"}
        </span>
        {hasActiveFilters && (
          <Button
            className="mt-3"
            size="sm"
            type="button"
            variant="outline"
            onClick={onClearFilters}
          >
            清除全部条件
          </Button>
        )}
      </KnowledgeState>
    );
  }

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between gap-3">
        <h2 className="text-sm font-medium text-muted-foreground">
          知识列表
        </h2>
        {error && (
          <span className="text-xs text-destructive" role="alert">
            刷新失败：{formatUiError(error)}
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
  const isPending = acceptMutation.isPending || archiveMutation.isPending;
  const mutationError = acceptMutation.error ?? archiveMutation.error;
  const hasLongContent =
    node.content.length > 360 || node.content.split("\n").length > 6;

  function acceptNode() {
    archiveMutation.reset();
    acceptMutation.mutate(node.id);
  }

  function archiveNode() {
    acceptMutation.reset();
    archiveMutation.mutate(node.id);
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
            <dt className="inline font-medium">创建于 </dt>
            <dd className="inline">
              <time dateTime={node.createdAt}>
                {formatDateTime(node.createdAt)}
              </time>
            </dd>
          </div>
          <div>
            <dt className="inline font-medium">更新于 </dt>
            <dd className="inline">
              <time dateTime={node.updatedAt}>
                {formatDateTime(node.updatedAt)}
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
            {isExpanded ? "收起内容" : "展开全文"}
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
              {acceptMutation.isPending ? "正在收录…" : "收录"}
            </Button>
            <Button
              size="sm"
              type="button"
              variant="outline"
              disabled={isPending}
              onClick={archiveNode}
            >
              {archiveMutation.isPending ? "正在归档…" : "归档"}
            </Button>
          </div>

          <div className="mt-2 min-h-5 text-xs" aria-live="polite">
            {mutationError && (
              <span className="text-destructive" role="alert">
                {formatUiError(mutationError)}
              </span>
            )}
          </div>
        </div>
      )}

      {node.status === "accepted" && (
        <KnowledgeExportStatus knowledgeId={node.id} />
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
    <StatusBadge tone="violet">
      {knowledgeTypeLabel(knowledgeType)}
    </StatusBadge>
  );
}

function KnowledgeStatusLabel({
  status,
}: {
  status: KnowledgeNodeDto["status"];
}) {
  const tone =
    status === "proposed"
      ? "amber"
      : status === "accepted"
        ? "green"
        : "gray";

  return (
    <StatusBadge tone={tone}>
      {knowledgeStatusLabel(status)}
    </StatusBadge>
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
