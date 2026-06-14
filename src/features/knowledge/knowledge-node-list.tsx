import { useKnowledgeNodes } from "@/features/knowledge/knowledge-queries";
import type { KnowledgeNodeDto } from "@/types/knowledge";

export function KnowledgeNodeList() {
  const knowledgeQuery = useKnowledgeNodes();

  if (knowledgeQuery.isPending) {
    return <KnowledgeState>Loading knowledge...</KnowledgeState>;
  }

  if (knowledgeQuery.isError) {
    return (
      <KnowledgeState tone="error">
        Could not load knowledge: {knowledgeQuery.error.message}
      </KnowledgeState>
    );
  }

  if (knowledgeQuery.data.length === 0) {
    return (
      <KnowledgeState>
        No knowledge nodes yet. Create the first one above.
      </KnowledgeState>
    );
  }

  return (
    <div className="space-y-3">
      <h2 className="text-sm font-medium text-muted-foreground">
        Knowledge nodes
      </h2>
      <ul className="space-y-3">
        {knowledgeQuery.data.map((node) => (
          <KnowledgeNodeItem key={node.id} node={node} />
        ))}
      </ul>
    </div>
  );
}

function KnowledgeNodeItem({ node }: { node: KnowledgeNodeDto }) {
  return (
    <li className="rounded-lg border bg-background p-4">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h3 className="font-semibold">{node.title}</h3>
          <p className="mt-1 text-xs capitalize text-muted-foreground">
            {node.knowledgeType} · {node.status}
          </p>
        </div>
        <time
          className="text-xs text-muted-foreground"
          dateTime={node.createdAt}
        >
          {formatCreatedAt(node.createdAt)}
        </time>
      </div>
      <p className="mt-3 whitespace-pre-wrap break-words text-sm">
        {node.content}
      </p>
    </li>
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

function formatCreatedAt(createdAt: string) {
  const date = new Date(createdAt);

  if (Number.isNaN(date.getTime())) {
    return createdAt;
  }

  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(date);
}
