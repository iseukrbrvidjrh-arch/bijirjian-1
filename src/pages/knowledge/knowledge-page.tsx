import { useState } from "react";

import { KnowledgeCreateForm } from "@/features/knowledge/knowledge-create-form";
import { KnowledgeFilterBar } from "@/features/knowledge/knowledge-filter-bar";
import { KnowledgeNodeList } from "@/features/knowledge/knowledge-node-list";
import { useKnowledgeNodes } from "@/features/knowledge/knowledge-queries";
import type {
  KnowledgeListFilters,
  KnowledgeStatus,
  KnowledgeType,
} from "@/types/knowledge";

const DEFAULT_KNOWLEDGE_LIMIT = 50;

export function KnowledgePage() {
  const [filters, setFilters] = useState<KnowledgeListFilters>({
    limit: DEFAULT_KNOWLEDGE_LIMIT,
  });
  const knowledgeQuery = useKnowledgeNodes(filters);
  const hasActiveFilters = Boolean(
    filters.status || filters.knowledgeType || filters.query,
  );

  function setStatus(status?: KnowledgeStatus) {
    setFilters((current) => ({ ...current, status }));
  }

  function setKnowledgeType(knowledgeType?: KnowledgeType) {
    setFilters((current) => ({ ...current, knowledgeType }));
  }

  function setQuery(query?: string) {
    const normalizedQuery = query?.trim();
    setFilters((current) => ({
      ...current,
      query: normalizedQuery || undefined,
    }));
  }

  function clearFilters() {
    setFilters({ limit: DEFAULT_KNOWLEDGE_LIMIT });
  }

  return (
    <section className="mx-auto max-w-4xl">
      <div>
        <h1 className="text-2xl font-semibold">知识库</h1>
        <p className="mt-2 text-sm text-muted-foreground">
          创建、审核和浏览保存在本地的结构化知识。
        </p>
      </div>

      <div className="mt-6 space-y-6">
        <KnowledgeCreateForm />
        <KnowledgeFilterBar
          status={filters.status}
          knowledgeType={filters.knowledgeType}
          query={filters.query}
          resultCount={knowledgeQuery.data?.length ?? 0}
          isRefreshing={
            knowledgeQuery.isFetching && !knowledgeQuery.isPending
          }
          onStatusChange={setStatus}
          onKnowledgeTypeChange={setKnowledgeType}
          onQueryChange={setQuery}
          onRefresh={() => void knowledgeQuery.refetch()}
        />
        <KnowledgeNodeList
          nodes={knowledgeQuery.data ?? []}
          isPending={knowledgeQuery.isPending}
          error={knowledgeQuery.error}
          hasActiveFilters={hasActiveFilters}
          onRetry={() => void knowledgeQuery.refetch()}
          onClearFilters={clearFilters}
        />
      </div>
    </section>
  );
}
