import { useState, type FormEvent } from "react";

import { Button } from "@/components/ui/button";
import { useCaptureTextSource } from "@/features/capture/source-queries";
import { formatUiError } from "@/lib/display";

export function CaptureForm() {
  const [rawContent, setRawContent] = useState("");
  const captureMutation = useCaptureTextSource();
  const isEmpty = rawContent.trim().length === 0;
  const characterCount = Array.from(rawContent).length;

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (isEmpty || captureMutation.isPending) {
      return;
    }

    captureMutation.mutate(rawContent, {
      onSuccess: () => {
        setRawContent("");
      },
    });
  }

  return (
    <form
      className="rounded-lg border bg-background p-4"
      onSubmit={handleSubmit}
    >
      <label className="text-sm font-medium" htmlFor="capture-content">
        添加文字内容
      </label>
      <textarea
        id="capture-content"
        className="mt-2 min-h-32 w-full resize-y rounded-md border bg-transparent px-3 py-2 text-sm outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
        value={rawContent}
        onChange={(event) => setRawContent(event.target.value)}
        placeholder="写下想法、笔记或摘录…"
        disabled={captureMutation.isPending}
      />
      <p
        className="mt-1 text-right text-xs text-muted-foreground"
        aria-live="polite"
      >
        已输入 {characterCount} 字
      </p>

      <div className="mt-3 flex items-center justify-between gap-4">
        <div aria-live="polite">
          {captureMutation.isError && (
            <p className="text-sm text-destructive" role="alert">
              {formatUiError(captureMutation.error, "保存失败，请稍后重试。")}
            </p>
          )}
        </div>

        <Button
          type="submit"
          disabled={isEmpty || captureMutation.isPending}
        >
          {captureMutation.isPending ? "正在保存…" : "保存"}
        </Button>
      </div>
    </form>
  );
}
