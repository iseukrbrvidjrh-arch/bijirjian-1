import { useEffect, useState, type FormEvent } from "react";

import { Button } from "@/components/ui/button";

interface InboxSearchBarProps {
  query?: string;
  isRefreshing: boolean;
  onQueryChange: (query?: string) => void;
  onRefresh: () => void;
}

export function InboxSearchBar({
  query,
  isRefreshing,
  onQueryChange,
  onRefresh,
}: InboxSearchBarProps) {
  const [draftQuery, setDraftQuery] = useState(query ?? "");

  useEffect(() => {
    setDraftQuery(query ?? "");
  }, [query]);

  function submitSearch(event: FormEvent<HTMLFormElement>) {
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
            htmlFor="inbox-search"
          >
            搜索收集箱
          </label>
          <input
            id="inbox-search"
            className="mt-1.5 h-9 w-full rounded-md border bg-background px-3 text-sm outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
            placeholder="搜索内容中的关键词…"
            type="search"
            value={draftQuery}
            onChange={(event) => setDraftQuery(event.target.value)}
          />
        </div>
        <Button type="submit">搜索</Button>
        {(query || draftQuery) && (
          <Button
            type="button"
            variant="outline"
            onClick={clearSearch}
          >
            清空搜索
          </Button>
        )}
        <Button
          type="button"
          variant="outline"
          disabled={isRefreshing}
          onClick={onRefresh}
        >
          {isRefreshing ? "正在刷新…" : "刷新"}
        </Button>
      </form>
    </section>
  );
}
