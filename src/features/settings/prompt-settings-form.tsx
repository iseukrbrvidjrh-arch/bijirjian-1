import { useState } from "react";
import { CheckCircle2, FileText } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  useCreatePromptVersion,
  useDefaultPrompt,
  usePromptVersions,
  useSetActivePromptVersion,
} from "@/features/settings/prompt-queries";

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
        `Version ${version.version} created. It is not active yet.`,
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
      setStatusMessage(`Version ${version} is now active.`);
      activateMutation.reset();
    } catch {
      // Mutation state renders the error.
    }
  }

  if (promptQuery.isPending || versionsQuery.isPending) {
    return <PromptState>Loading prompt settings...</PromptState>;
  }

  if (promptQuery.isError || versionsQuery.isError) {
    const error = promptQuery.error ?? versionsQuery.error;
    return (
      <PromptState tone="error">
        Could not load prompt settings: {errorMessage(error)}
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
          <h2 className="font-semibold">Prompt Settings</h2>
          <p className="mt-1 text-sm text-muted-foreground">
            Manage immutable versions of the built-in source summary
            prompt.
          </p>
        </div>
      </div>

      <div className="mt-6 grid gap-3 rounded-md border bg-muted/30 p-4 sm:grid-cols-2">
        <div>
          <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
            Prompt
          </p>
          <p className="mt-1 text-sm font-medium">{prompt.name}</p>
        </div>
        <div>
          <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
            Active version
          </p>
          <p className="mt-1 text-sm font-medium">
            Version {prompt.activeVersion.version}
          </p>
        </div>
      </div>

      <form className="mt-5" onSubmit={createVersion}>
        <label className="text-sm font-medium" htmlFor="prompt-content">
          New prompt version
        </label>
        <textarea
          id="prompt-content"
          rows={6}
          value={promptContent}
          onChange={(event) => setPromptContent(event.target.value)}
          className="mt-2 w-full resize-y rounded-md border bg-background px-3 py-2 text-sm outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
          placeholder="Enter the complete content for a new prompt version."
        />
        <div className="mt-3 flex justify-end">
          <Button
            type="submit"
            disabled={isMutating || promptContent.trim().length === 0}
          >
            {createMutation.isPending
              ? "Creating..."
              : "Create new version"}
          </Button>
        </div>
      </form>

      <div className="mt-5 min-h-5 text-sm" aria-live="polite">
        {statusMessage && (
          <span className="inline-flex items-center gap-1.5">
            <CheckCircle2 className="size-4" />
            {statusMessage}
          </span>
        )}
        {createMutation.isError && (
          <span className="text-destructive" role="alert">
            {errorMessage(createMutation.error)}
          </span>
        )}
        {activateMutation.isError && (
          <span className="text-destructive" role="alert">
            {errorMessage(activateMutation.error)}
          </span>
        )}
      </div>

      <div className="mt-5 border-t pt-5">
        <h3 className="text-sm font-semibold">Version history</h3>
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
                      Version {version.version}
                    </p>
                    <p className="mt-1 text-xs text-muted-foreground">
                      {formatDate(version.createdAt)}
                    </p>
                  </div>
                  {isActive ? (
                    <span className="rounded-full bg-muted px-2.5 py-1 text-xs font-medium">
                      Active
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
                      {isActivating ? "Activating..." : "Set active"}
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

function formatDate(value: string) {
  const date = new Date(value);
  return Number.isNaN(date.getTime())
    ? value
    : new Intl.DateTimeFormat(undefined, {
        dateStyle: "medium",
        timeStyle: "short",
      }).format(date);
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
