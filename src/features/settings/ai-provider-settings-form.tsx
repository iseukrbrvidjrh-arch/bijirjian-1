import { useEffect, useState } from "react";
import { CheckCircle2, KeyRound } from "lucide-react";
import { useForm } from "react-hook-form";

import { Button } from "@/components/ui/button";
import {
  useAiProviderSettings,
  useSaveAiProviderSettings,
  useTestAiProviderConnection,
} from "@/features/settings/ai-provider-queries";
import { formatUiError } from "@/lib/display";
import type {
  AiProviderModel,
  AiProviderType,
  SaveAiProviderSettingsInput,
} from "@/types/ai-provider";

interface ProviderFormValues {
  providerType: AiProviderType;
  defaultModel: AiProviderModel;
  apiKey: string;
}

export function AiProviderSettingsForm() {
  const settingsQuery = useAiProviderSettings();
  const saveMutation = useSaveAiProviderSettings();
  const testMutation = useTestAiProviderConnection();
  const [saveMessage, setSaveMessage] = useState<string>();
  const [saveError, setSaveError] = useState<string>();
  const {
    register,
    handleSubmit,
    resetField,
    setValue,
    formState: { errors },
  } = useForm<ProviderFormValues>({
    defaultValues: {
      providerType: "deepseek",
      defaultModel: "deepseek-v4-flash",
      apiKey: "",
    },
  });

  useEffect(() => {
    if (settingsQuery.data) {
      setValue("providerType", settingsQuery.data.providerType);
      setValue("defaultModel", settingsQuery.data.defaultModel);
    }
  }, [setValue, settingsQuery.data]);

  async function saveSettings(values: ProviderFormValues) {
    setSaveMessage(undefined);
    setSaveError(undefined);
    testMutation.reset();

    const input: SaveAiProviderSettingsInput = {
      providerType: values.providerType,
      defaultModel: values.defaultModel,
      apiKey: values.apiKey.trim() || undefined,
    };

    try {
      await saveMutation.mutateAsync(input);
      resetField("apiKey");
      setSaveMessage("AI 服务设置已安全保存。");
      saveMutation.reset();
    } catch (error) {
      setSaveError(formatUiError(error));
      saveMutation.reset();
    }
  }

  function testConnection() {
    setSaveMessage(undefined);
    setSaveError(undefined);
    testMutation.mutate();
  }

  if (settingsQuery.isPending) {
    return <SettingsState>正在加载 AI 服务设置…</SettingsState>;
  }

  if (settingsQuery.isError) {
    return (
      <SettingsState tone="error">
        AI 服务设置加载失败：
        {formatUiError(settingsQuery.error)}
      </SettingsState>
    );
  }

  const settings = settingsQuery.data;
  const hasSavedApiKey = settings?.hasApiKey ?? false;

  return (
    <section className="rounded-lg border bg-background p-5">
      <div className="flex items-start gap-3">
        <div className="rounded-md bg-muted p-2">
          <KeyRound className="size-4" />
        </div>
        <div>
          <h2 className="font-semibold">AI 服务</h2>
          <p className="mt-1 text-sm text-muted-foreground">
            配置用于生成总结的 DeepSeek。API Key 只保存在 macOS
            钥匙串中，保存后不会再次显示。
          </p>
        </div>
      </div>

      <form
        className="mt-6 space-y-5"
        onSubmit={handleSubmit(saveSettings)}
      >
        <div>
          <label className="text-sm font-medium" htmlFor="provider-type">
            服务商
          </label>
          <select
            id="provider-type"
            className="mt-2 h-9 w-full rounded-md border bg-background px-3 text-sm outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
            {...register("providerType")}
          >
            <option value="deepseek">DeepSeek</option>
          </select>
        </div>

        <div>
          <label className="text-sm font-medium" htmlFor="default-model">
            默认模型
          </label>
          <select
            id="default-model"
            className="mt-2 h-9 w-full rounded-md border bg-background px-3 text-sm outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
            {...register("defaultModel")}
          >
            <option value="deepseek-v4-flash">
              DeepSeek V4 Flash
            </option>
            <option value="deepseek-v4-pro">DeepSeek V4 Pro</option>
          </select>
          <p className="mt-1 text-xs text-muted-foreground">
            生成 AI 总结时默认使用这个模型。
          </p>
        </div>

        <div>
          <div className="flex items-center justify-between gap-3">
            <label className="text-sm font-medium" htmlFor="provider-api-key">
              API Key
            </label>
            <span className="text-xs text-muted-foreground">
              {hasSavedApiKey ? "已保存到钥匙串" : "尚未配置"}
            </span>
          </div>
          <input
            id="provider-api-key"
            type="password"
            autoComplete="new-password"
            className="mt-2 h-9 w-full rounded-md border bg-background px-3 text-sm outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
            placeholder={
              hasSavedApiKey
                ? "留空可继续使用已保存的 API Key"
                : "请输入 DeepSeek API Key"
            }
            {...register("apiKey", {
              validate: (value) =>
                hasSavedApiKey ||
                value.trim().length > 0 ||
                "首次保存时必须填写 API Key。",
            })}
          />
          {errors.apiKey && (
            <p className="mt-1 text-xs text-destructive" role="alert">
              {errors.apiKey.message}
            </p>
          )}
        </div>

        <div className="flex flex-wrap items-center justify-between gap-3 border-t pt-4">
          <div className="text-sm" aria-live="polite">
            {saveMessage && (
              <span className="inline-flex items-center gap-1.5 text-emerald-700">
                <CheckCircle2 className="size-4" />
                {saveMessage}
              </span>
            )}
            {saveError && (
              <span className="text-destructive" role="alert">
                {saveError}
              </span>
            )}
            {testMutation.isSuccess && (
              <span className="inline-flex items-center gap-1.5 text-emerald-700">
                <CheckCircle2 className="size-4" />
              DeepSeek 连接成功。
              </span>
            )}
            {testMutation.isError && (
              <span className="text-destructive" role="alert">
              {formatUiError(
                testMutation.error,
                "连接测试失败，请检查 API Key 和网络。",
              )}
              </span>
            )}
          </div>

          <div className="flex items-center gap-2">
            <Button
              type="button"
              variant="outline"
              disabled={
                !settings?.hasApiKey ||
                saveMutation.isPending ||
                testMutation.isPending
              }
              onClick={testConnection}
            >
              {testMutation.isPending
                ? "正在测试…"
                : "测试连接"}
            </Button>
            <Button
              type="submit"
              disabled={
                saveMutation.isPending || testMutation.isPending
              }
            >
              {saveMutation.isPending ? "正在保存…" : "保存设置"}
            </Button>
          </div>
        </div>
      </form>
    </section>
  );
}

function SettingsState({
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
