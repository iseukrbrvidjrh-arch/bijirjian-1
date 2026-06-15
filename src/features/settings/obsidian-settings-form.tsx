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
import { formatUiError } from "@/lib/display";
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
      setSaveMessage("Obsidian 仓库路径已保存。");
      saveMutation.reset();
    } catch {
      // Mutation state renders the error below.
    }
  }

  if (settingsQuery.isPending) {
    return <ObsidianState>正在加载 Obsidian 设置…</ObsidianState>;
  }

  if (settingsQuery.isError) {
    return (
      <ObsidianState tone="error">
        Obsidian 设置加载失败：
        {formatUiError(settingsQuery.error)}
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
          <h2 className="font-semibold">Obsidian 仓库</h2>
          <p className="mt-1 text-sm text-muted-foreground">
            保存当前工作区对应的本地仓库路径。这里不会扫描或读取仓库中的笔记。
          </p>
        </div>
      </div>

      <form
        className="mt-6 space-y-5"
        onSubmit={handleSubmit(saveSettings)}
      >
        <div>
          <label className="text-sm font-medium" htmlFor="obsidian-vault-path">
            仓库路径
          </label>
          <input
            id="obsidian-vault-path"
            className="mt-2 h-9 w-full rounded-md border bg-background px-3 text-sm outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
            placeholder="/Users/你的名字/Documents/我的知识库"
            disabled={saveMutation.isPending}
            {...register("vaultPath", {
              validate: (value) =>
                value.trim().length > 0 || "请填写 Obsidian 仓库路径。",
            })}
          />
          <p className="mt-1 text-xs text-muted-foreground">
            请输入已经存在的本地文件夹路径。路径会保存在当前工作区的 SQLite 中。
          </p>
          {errors.vaultPath && (
            <p className="mt-1 text-xs text-destructive" role="alert">
              {errors.vaultPath.message}
            </p>
          )}
        </div>

        {settings && !settings.hasObsidianDirectory && (
          <div
            className="flex items-start gap-2 rounded-md border border-amber-200 bg-amber-50 p-3 text-sm text-amber-800"
            role="status"
          >
            <AlertTriangle className="mt-0.5 size-4 shrink-0" />
            <span>
              这个文件夹中还没有 `.obsidian` 目录。路径可以保存，但
              Obsidian 可能尚未将它初始化为仓库。
            </span>
          </div>
        )}

        <div className="flex flex-wrap items-center justify-between gap-3 border-t pt-4">
          <div className="text-sm" aria-live="polite">
            {saveMessage && (
              <span className="inline-flex items-center gap-1.5 text-emerald-700">
                <CheckCircle2 className="size-4" />
                {saveMessage}
              </span>
            )}
            {saveMutation.isError && (
              <span className="text-destructive" role="alert">
                {formatUiError(
                  saveMutation.error,
                  "保存失败，请确认路径存在且为文件夹。",
                )}
              </span>
            )}
          </div>

          <Button type="submit" disabled={saveMutation.isPending}>
            {saveMutation.isPending ? "正在保存…" : "保存仓库路径"}
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
