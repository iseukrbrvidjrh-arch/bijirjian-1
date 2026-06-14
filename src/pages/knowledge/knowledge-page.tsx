import { KnowledgeCreateForm } from "@/features/knowledge/knowledge-create-form";
import { KnowledgeNodeList } from "@/features/knowledge/knowledge-node-list";

export function KnowledgePage() {
  return (
    <section className="mx-auto max-w-3xl">
      <div>
        <h1 className="text-2xl font-semibold">Knowledge</h1>
        <p className="mt-2 text-sm text-muted-foreground">
          Create and browse structured knowledge stored locally.
        </p>
      </div>

      <div className="mt-6 space-y-6">
        <KnowledgeCreateForm />
        <KnowledgeNodeList />
      </div>
    </section>
  );
}
