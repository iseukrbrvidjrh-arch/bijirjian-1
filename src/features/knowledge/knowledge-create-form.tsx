import { useForm } from "react-hook-form";

import { Button } from "@/components/ui/button";
import { useCreateKnowledgeNode } from "@/features/knowledge/knowledge-queries";
import type {
  CreateKnowledgeNodeInput,
  KnowledgeType,
} from "@/types/knowledge";

const KNOWLEDGE_TYPES: Array<{
  value: KnowledgeType;
  label: string;
}> = [
  { value: "concept", label: "Concept" },
  { value: "tool", label: "Tool" },
  { value: "project", label: "Project" },
  { value: "question", label: "Question" },
  { value: "solution", label: "Solution" },
  { value: "insight", label: "Insight" },
  { value: "resource", label: "Resource" },
  { value: "person", label: "Person" },
];

export function KnowledgeCreateForm() {
  const createMutation = useCreateKnowledgeNode();
  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<CreateKnowledgeNodeInput>({
    defaultValues: {
      title: "",
      content: "",
      knowledgeType: "concept",
    },
  });

  async function createNode(values: CreateKnowledgeNodeInput) {
    try {
      await createMutation.mutateAsync({
        title: values.title.trim(),
        content: values.content.trim(),
        knowledgeType: values.knowledgeType,
      });
      reset();
    } catch {
      // Mutation state renders the error below the form.
    }
  }

  return (
    <form
      className="rounded-lg border bg-background p-5"
      onSubmit={handleSubmit(createNode)}
    >
      <div>
        <h2 className="font-semibold">Create knowledge</h2>
        <p className="mt-1 text-sm text-muted-foreground">
          Add a structured knowledge node to the local database.
        </p>
      </div>

      <div className="mt-5 space-y-4">
        <div>
          <label className="text-sm font-medium" htmlFor="knowledge-title">
            Title
          </label>
          <input
            id="knowledge-title"
            className="mt-2 h-9 w-full rounded-md border bg-background px-3 text-sm outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
            placeholder="Name this knowledge node"
            disabled={createMutation.isPending}
            {...register("title", {
              validate: (value) =>
                value.trim().length > 0 || "Title is required.",
            })}
          />
          {errors.title && (
            <p className="mt-1 text-xs text-destructive" role="alert">
              {errors.title.message}
            </p>
          )}
        </div>

        <div>
          <label className="text-sm font-medium" htmlFor="knowledge-type">
            Type
          </label>
          <select
            id="knowledge-type"
            className="mt-2 h-9 w-full rounded-md border bg-background px-3 text-sm outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
            disabled={createMutation.isPending}
            {...register("knowledgeType")}
          >
            {KNOWLEDGE_TYPES.map((knowledgeType) => (
              <option key={knowledgeType.value} value={knowledgeType.value}>
                {knowledgeType.label}
              </option>
            ))}
          </select>
        </div>

        <div>
          <label className="text-sm font-medium" htmlFor="knowledge-content">
            Content
          </label>
          <textarea
            id="knowledge-content"
            className="mt-2 min-h-32 w-full resize-y rounded-md border bg-transparent px-3 py-2 text-sm outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
            placeholder="Write the knowledge content..."
            disabled={createMutation.isPending}
            {...register("content", {
              validate: (value) =>
                value.trim().length > 0 || "Content is required.",
            })}
          />
          {errors.content && (
            <p className="mt-1 text-xs text-destructive" role="alert">
              {errors.content.message}
            </p>
          )}
        </div>
      </div>

      <div className="mt-4 flex items-center justify-between gap-4">
        <div className="text-sm" aria-live="polite">
          {createMutation.isSuccess && (
            <span>Knowledge node created.</span>
          )}
          {createMutation.isError && (
            <span className="text-destructive" role="alert">
              {createMutation.error.message}
            </span>
          )}
        </div>

        <Button type="submit" disabled={createMutation.isPending}>
          {createMutation.isPending ? "Creating..." : "Create"}
        </Button>
      </div>
    </form>
  );
}
