import { useEffect, useState } from "react";

import { Button } from "@/components/ui/button";
import type {
  KnowledgeStatus,
  KnowledgeType,
} from "@/types/knowledge";

const STATUS_OPTIONS: Array<{
  value: KnowledgeStatus | "";
  label: string;
}> = [
  { value: "", label: "All" },
  { value: "proposed", label: "Proposed" },
  { value: "accepted", label: "Accepted" },
  { value: "archived", label: "Archived" },
];

const TYPE_OPTIONS: Array<{
  value: KnowledgeType | "";
  label: string;
}> = [
  { value: "", label: "All" },
  { value: "concept", label: "Concept" },
  { value: "tool", label: "Tool" },
  { value: "project", label: "Project" },
  { value: "question", label: "Question" },
  { value: "solution", label: "Solution" },
  { value: "insight", label: "Insight" },
  { value: "resource", label: "Resource" },
  { value: "person", label: "Person" },
];

interface KnowledgeFilterBarProps {
  status?: KnowledgeStatus;
  knowledgeType?: KnowledgeType;
  query?: string;
  resultCount: number;
  isRefreshing: boolean;
  onStatusChange: (status?: KnowledgeStatus) => void;
  onKnowledgeTypeChange: (knowledgeType?: KnowledgeType) => void;
  onQueryChange: (query?: string) => void;
  onRefresh: () => void;
}

export function KnowledgeFilterBar({
  status,
  knowledgeType,
  query,
  resultCount,
  isRefreshing,
  onStatusChange,
  onKnowledgeTypeChange,
  onQueryChange,
  onRefresh,
}: KnowledgeFilterBarProps) {
  const [draftQuery, setDraftQuery] = useState(query ?? "");

  useEffect(() => {
    setDraftQuery(query ?? "");
  }, [query]);

  function submitSearch(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const normalizedQuery = draftQuery.trim();
    onQueryChange(normalizedQuery || undefined);
  }

  function clearSearch() {
    setDraftQuery("");
    onQueryChange(undefined);
  }

  return (
    <section className="rounded-lg border bg-background p-4">
      <form
        className="flex flex-wrap items-end gap-2"
        onSubmit={submitSearch}
      >
        <div className="min-w-60 flex-1">
          <label
            className="text-xs font-medium text-muted-foreground"
            htmlFor="knowledge-search"
          >
            Search
          </label>
          <input
            id="knowledge-search"
            className="mt-1.5 h-9 w-full rounded-md border bg-background px-3 text-sm outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
            placeholder="Search title or content..."
            type="search"
            value={draftQuery}
            onChange={(event) => setDraftQuery(event.target.value)}
          />
        </div>
        <Button type="submit">Search</Button>
        {(query || draftQuery) && (
          <Button
            type="button"
            variant="outline"
            onClick={clearSearch}
          >
            Clear search
          </Button>
        )}
      </form>

      <div className="mt-4 flex flex-wrap items-end gap-3">
        <div className="min-w-40 flex-1">
          <label
            className="text-xs font-medium text-muted-foreground"
            htmlFor="knowledge-status-filter"
          >
            Status
          </label>
          <select
            id="knowledge-status-filter"
            className="mt-1.5 h-9 w-full rounded-md border bg-background px-3 text-sm outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
            value={status ?? ""}
            onChange={(event) =>
              onStatusChange(
                event.target.value
                  ? (event.target.value as KnowledgeStatus)
                  : undefined,
              )
            }
          >
            {STATUS_OPTIONS.map((option) => (
              <option key={option.value || "all"} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        </div>

        <div className="min-w-40 flex-1">
          <label
            className="text-xs font-medium text-muted-foreground"
            htmlFor="knowledge-type-filter"
          >
            Type
          </label>
          <select
            id="knowledge-type-filter"
            className="mt-1.5 h-9 w-full rounded-md border bg-background px-3 text-sm outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
            value={knowledgeType ?? ""}
            onChange={(event) =>
              onKnowledgeTypeChange(
                event.target.value
                  ? (event.target.value as KnowledgeType)
                  : undefined,
              )
            }
          >
            {TYPE_OPTIONS.map((option) => (
              <option key={option.value || "all"} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        </div>

        <Button
          type="button"
          variant="outline"
          disabled={isRefreshing}
          onClick={onRefresh}
        >
          {isRefreshing ? "Refreshing..." : "Refresh"}
        </Button>
      </div>

      <p className="mt-3 text-xs text-muted-foreground" aria-live="polite">
        {resultCount} {resultCount === 1 ? "result" : "results"}
      </p>
    </section>
  );
}
