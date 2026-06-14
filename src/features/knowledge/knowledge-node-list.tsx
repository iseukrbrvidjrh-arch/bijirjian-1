import { Button } from "@/components/ui/button";
import {
  useAcceptKnowledgeNode,
  useArchiveKnowledgeNode,
  useKnowledgeNodes,
} from "@/features/knowledge/knowledge-queries";
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
  const acceptMutation = useAcceptKnowledgeNode();
  const archiveMutation = useArchiveKnowledgeNode();
  const isPending =
    acceptMutation.isPending || archiveMutation.isPending;
  const mutationError =
    acceptMutation.error ?? archiveMutation.error;

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
        <div>
          <h3 className="font-semibold">{node.title}</h3>
          <div className="mt-1 flex flex-wrap items-center gap-2 text-xs">
            <span className="capitalize text-muted-foreground">
              {node.knowledgeType}
            </span>
            <KnowledgeStatusLabel status={node.status} />
          </div>
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

      {node.status === "proposed" && (
        <div className="mt-4">
          <div className="flex flex-wrap items-center gap-2">
            <Button
              size="sm"
              type="button"
              disabled={isPending}
              onClick={acceptNode}
            >
              {acceptMutation.isPending ? "Accepting…" : "Accept"}
            </Button>
            <Button
              size="sm"
              type="button"
              variant="outline"
              disabled={isPending}
              onClick={archiveNode}
            >
              {archiveMutation.isPending ? "Archiving…" : "Archive"}
            </Button>
          </div>

          <div className="mt-2 min-h-5 text-xs" aria-live="polite">
            {mutationError && (
              <span className="text-destructive" role="alert">
                {mutationError.message}
              </span>
            )}
          </div>
        </div>
      )}
    </li>
  );
}

function KnowledgeStatusLabel({
  status,
}: {
  status: KnowledgeNodeDto["status"];
}) {
  const tone =
    status === "proposed"
      ? "border-amber-500/30 bg-amber-500/10 text-amber-700 dark:text-amber-300"
      : status === "accepted"
        ? "border-emerald-500/30 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300"
        : "border-border bg-muted text-muted-foreground";

  return (
    <span
      className={`rounded-full border px-2 py-0.5 font-medium capitalize ${tone}`}
    >
      {status}
    </span>
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
