import { useEffect, useState } from "react";
import {
  AlertTriangle,
  CheckCircle2,
  FolderCog,
} from "lucide-react";
import { useForm } from "react-hook-form";

import { Button } from "@/components/ui/button";
import {
  useObsidianSettings,
  useSaveObsidianSettings,
} from "@/features/settings/obsidian-settings-queries";
import type { SaveObsidianSettingsInput } from "@/types/obsidian-settings";

export function ObsidianSettingsForm() {
  const settingsQuery = useObsidianSettings();
  const saveMutation = useSaveObsidianSettings();
  const [saveMessage, setSaveMessage] = useState<string>();
  const {
    register,
    handleSubmit,
    setValue,
    formState: { errors },
  } = useForm<SaveObsidianSettingsInput>({
    defaultValues: {
      vaultPath: "",
    },
  });

  useEffect(() => {
    if (settingsQuery.data) {
      setValue("vaultPath", settingsQuery.data.vaultPath);
    }
  }, [setValue, settingsQuery.data]);

  async function saveSettings(values: SaveObsidianSettingsInput) {
    setSaveMessage(undefined);
    saveMutation.reset();

    try {
      await saveMutation.mutateAsync({
        vaultPath: values.vaultPath.trim(),
      });
      setSaveMessage("Obsidian Vault path saved.");
      saveMutation.reset();
    } catch {
      // Mutation state renders the error below.
    }
  }

  if (settingsQuery.isPending) {
    return <ObsidianState>Loading Obsidian settings...</ObsidianState>;
  }

  if (settingsQuery.isError) {
    return (
      <ObsidianState tone="error">
        Could not load Obsidian settings:{" "}
        {errorMessage(settingsQuery.error)}
      </ObsidianState>
    );
  }

  const settings = settingsQuery.data;

  return (
    <section className="rounded-lg border bg-background p-5">
      <div className="flex items-start gap-3">
        <div className="rounded-md bg-muted p-2">
          <FolderCog className="size-4" />
        </div>
        <div>
          <h2 className="font-semibold">Obsidian Vault</h2>
          <p className="mt-1 text-sm text-muted-foreground">
            Store the local Vault path for the current workspace. This
            setting does not read, scan, or write Vault files.
          </p>
        </div>
      </div>

      <form
        className="mt-6 space-y-5"
        onSubmit={handleSubmit(saveSettings)}
      >
        <div>
          <label className="text-sm font-medium" htmlFor="obsidian-vault-path">
            Vault path
          </label>
          <input
            id="obsidian-vault-path"
            className="mt-2 h-9 w-full rounded-md border bg-background px-3 text-sm outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
            placeholder="/Users/you/Documents/My Vault"
            disabled={saveMutation.isPending}
            {...register("vaultPath", {
              validate: (value) =>
                value.trim().length > 0 || "Vault path is required.",
            })}
          />
          <p className="mt-1 text-xs text-muted-foreground">
            Enter an existing local directory. The path is stored in
            SQLite for this workspace.
          </p>
          {errors.vaultPath && (
            <p className="mt-1 text-xs text-destructive" role="alert">
              {errors.vaultPath.message}
            </p>
          )}
        </div>

        {settings && !settings.hasObsidianDirectory && (
          <div
            className="flex items-start gap-2 rounded-md border border-amber-500/30 bg-amber-500/10 p-3 text-sm text-amber-800 dark:text-amber-200"
            role="status"
          >
            <AlertTriangle className="mt-0.5 size-4 shrink-0" />
            <span>
              This directory does not currently contain a .obsidian
              folder. The path is saved, but Obsidian may not have
              initialized it as a Vault yet.
            </span>
          </div>
        )}

        <div className="flex flex-wrap items-center justify-between gap-3 border-t pt-4">
          <div className="text-sm" aria-live="polite">
            {saveMessage && (
              <span className="inline-flex items-center gap-1.5">
                <CheckCircle2 className="size-4" />
                {saveMessage}
              </span>
            )}
            {saveMutation.isError && (
              <span className="text-destructive" role="alert">
                {errorMessage(saveMutation.error)}
              </span>
            )}
          </div>

          <Button type="submit" disabled={saveMutation.isPending}>
            {saveMutation.isPending ? "Saving..." : "Save Vault path"}
          </Button>
        </div>
      </form>
    </section>
  );
}

function ObsidianState({
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

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
