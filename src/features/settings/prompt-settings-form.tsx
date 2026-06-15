import { useState } from "react";
import { CheckCircle2, FileText } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  useCreatePromptVersion,
  useDefaultPrompt,
  usePromptVersions,
  useSetActivePromptVersion,
} from "@/features/settings/prompt-queries";
import { formatDateTime, formatUiError } from "@/lib/display";

export function PromptSettingsForm() {
  const promptQuery = useDefaultPrompt();
  const versionsQuery = usePromptVersions();
  const createMutation = useCreatePromptVersion();
  const activateMutation = useSetActivePromptVersion();
  const [promptContent, setPromptContent] = useState("");
  const [statusMessage, setStatusMessage] = useState<string>();

  async function createVersion(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setStatusMessage(undefined);
    createMutation.reset();
    activateMutation.reset();

    try {
      const version = await createMutation.mutateAsync(promptContent);
      setPromptContent("");
      setStatusMessage(
        `版本 ${version.version} 已创建，尚未启用。`,
      );
      createMutation.reset();
    } catch {
      // Mutation state renders the error.
    }
  }

  async function setActiveVersion(versionId: string, version: number) {
    setStatusMessage(undefined);
    createMutation.reset();
    activateMutation.reset();

    try {
      await activateMutation.mutateAsync(versionId);
      setStatusMessage(`版本 ${version} 已设为当前版本。`);
      activateMutation.reset();
    } catch {
      // Mutation state renders the error.
    }
  }

  if (promptQuery.isPending || versionsQuery.isPending) {
    return <PromptState>正在加载提示词设置…</PromptState>;
  }

  if (promptQuery.isError || versionsQuery.isError) {
    const error = promptQuery.error ?? versionsQuery.error;
    return (
      <PromptState tone="error">
        提示词设置加载失败：{formatUiError(error)}
      </PromptState>
    );
  }

  const prompt = promptQuery.data;
  const versions = versionsQuery.data;
  const isMutating =
    createMutation.isPending || activateMutation.isPending;

  return (
    <section className="rounded-lg border bg-background p-5">
      <div className="flex items-start gap-3">
        <div className="rounded-md bg-muted p-2">
          <FileText className="size-4" />
        </div>
        <div>
          <h2 className="font-semibold">总结提示词</h2>
          <p className="mt-1 text-sm text-muted-foreground">
            管理内容总结所使用的提示词版本。历史版本创建后不会被修改。
          </p>
        </div>
      </div>

      <div className="mt-6 grid gap-3 rounded-md border bg-muted/30 p-4 sm:grid-cols-2">
        <div>
          <p className="text-xs font-medium tracking-wide text-muted-foreground">
            提示词
          </p>
          <p className="mt-1 text-sm font-medium">
            {prompt.promptKey === "source_summary"
              ? "内容总结"
              : prompt.name}
          </p>
        </div>
        <div>
          <p className="text-xs font-medium tracking-wide text-muted-foreground">
            当前版本
          </p>
          <p className="mt-1 text-sm font-medium">
            版本 {prompt.activeVersion.version}
          </p>
        </div>
      </div>

      <form className="mt-5" onSubmit={createVersion}>
        <label className="text-sm font-medium" htmlFor="prompt-content">
          新建提示词版本
        </label>
        <textarea
          id="prompt-content"
          rows={6}
          value={promptContent}
          onChange={(event) => setPromptContent(event.target.value)}
          className="mt-2 w-full resize-y rounded-md border bg-background px-3 py-2 text-sm outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
          placeholder="输入完整的提示词内容…"
        />
        <div className="mt-3 flex justify-end">
          <Button
            type="submit"
            disabled={isMutating || promptContent.trim().length === 0}
          >
            {createMutation.isPending
              ? "正在创建…"
              : "创建新版本"}
          </Button>
        </div>
      </form>

      <div className="mt-5 min-h-5 text-sm" aria-live="polite">
        {statusMessage && (
          <span className="inline-flex items-center gap-1.5 text-emerald-700">
            <CheckCircle2 className="size-4" />
            {statusMessage}
          </span>
        )}
        {createMutation.isError && (
          <span className="text-destructive" role="alert">
            {formatUiError(createMutation.error)}
          </span>
        )}
        {activateMutation.isError && (
          <span className="text-destructive" role="alert">
            {formatUiError(activateMutation.error)}
          </span>
        )}
      </div>

      <div className="mt-5 border-t pt-5">
        <h3 className="text-sm font-semibold">版本历史</h3>
        <div className="mt-3 space-y-3">
          {versions.map((version) => {
            const isActive = version.id === prompt.activeVersionId;
            const isActivating =
              activateMutation.isPending &&
              activateMutation.variables === version.id;

            return (
              <article
                key={version.id}
                className="rounded-md border p-4"
              >
                <div className="flex flex-wrap items-center justify-between gap-3">
                  <div>
                    <p className="text-sm font-medium">
                      版本 {version.version}
                    </p>
                    <p className="mt-1 text-xs text-muted-foreground">
                      {formatDateTime(version.createdAt)}
                    </p>
                  </div>
                  {isActive ? (
                    <span className="rounded-full bg-muted px-2.5 py-1 text-xs font-medium">
                      当前使用
                    </span>
                  ) : (
                    <Button
                      type="button"
                      size="sm"
                      variant="outline"
                      disabled={isMutating}
                      onClick={() =>
                        setActiveVersion(version.id, version.version)
                      }
                    >
                      {isActivating ? "正在切换…" : "设为当前版本"}
                    </Button>
                  )}
                </div>
                <p className="mt-3 whitespace-pre-wrap text-sm text-muted-foreground">
                  {version.promptContent}
                </p>
              </article>
            );
          })}
        </div>
      </div>
    </section>
  );
}

function PromptState({
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
