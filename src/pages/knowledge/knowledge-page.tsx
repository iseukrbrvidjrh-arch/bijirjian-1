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
    filters.status || filters.knowledgeType,
  );

  function setStatus(status?: KnowledgeStatus) {
    setFilters((current) => ({ ...current, status }));
  }

  function setKnowledgeType(knowledgeType?: KnowledgeType) {
    setFilters((current) => ({ ...current, knowledgeType }));
  }

  function clearFilters() {
    setFilters({ limit: DEFAULT_KNOWLEDGE_LIMIT });
  }

  return (
    <section className="mx-auto max-w-4xl">
      <div>
        <h1 className="text-2xl font-semibold">Knowledge</h1>
        <p className="mt-2 text-sm text-muted-foreground">
          Create, review, and browse structured knowledge stored locally.
        </p>
      </div>

      <div className="mt-6 space-y-6">
        <KnowledgeCreateForm />
        <KnowledgeFilterBar
          status={filters.status}
          knowledgeType={filters.knowledgeType}
          resultCount={knowledgeQuery.data?.length ?? 0}
          isRefreshing={
            knowledgeQuery.isFetching && !knowledgeQuery.isPending
          }
          onStatusChange={setStatus}
          onKnowledgeTypeChange={setKnowledgeType}
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
