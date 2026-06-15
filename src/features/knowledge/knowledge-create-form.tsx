import { useForm } from "react-hook-form";

import { Button } from "@/components/ui/button";
import { useCreateKnowledgeNode } from "@/features/knowledge/knowledge-queries";
import {
  formatUiError,
  knowledgeTypeLabel,
} from "@/lib/display";
import type {
  CreateKnowledgeNodeInput,
  KnowledgeType,
} from "@/types/knowledge";

const KNOWLEDGE_TYPE_VALUES: KnowledgeType[] = [
  "concept",
  "tool",
  "project",
  "question",
  "solution",
  "insight",
  "resource",
  "person",
];

const KNOWLEDGE_TYPES: Array<{
  value: KnowledgeType;
  label: string;
}> = KNOWLEDGE_TYPE_VALUES.map((value) => ({
  value,
  label: knowledgeTypeLabel(value),
}));

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
        <h2 className="font-semibold">手动添加知识</h2>
        <p className="mt-1 text-sm text-muted-foreground">
          将已经整理好的内容直接保存到本地知识库。
        </p>
      </div>

      <div className="mt-5 space-y-4">
        <div>
          <label className="text-sm font-medium" htmlFor="knowledge-title">
            标题
          </label>
          <input
            id="knowledge-title"
            className="mt-2 h-9 w-full rounded-md border bg-background px-3 text-sm outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
            placeholder="为这条知识起一个清晰的标题"
            disabled={createMutation.isPending}
            {...register("title", {
              validate: (value) =>
                value.trim().length > 0 || "请填写标题。",
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
            类型
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
            内容
          </label>
          <textarea
            id="knowledge-content"
            className="mt-2 min-h-32 w-full resize-y rounded-md border bg-transparent px-3 py-2 text-sm outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
            placeholder="写下完整的知识内容…"
            disabled={createMutation.isPending}
            {...register("content", {
              validate: (value) =>
                value.trim().length > 0 || "请填写知识内容。",
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
            <span className="text-emerald-700">知识已创建。</span>
          )}
          {createMutation.isError && (
            <span className="text-destructive" role="alert">
              {formatUiError(createMutation.error)}
            </span>
          )}
        </div>

        <Button type="submit" disabled={createMutation.isPending}>
          {createMutation.isPending ? "正在创建…" : "创建知识"}
        </Button>
      </div>
    </form>
  );
}
