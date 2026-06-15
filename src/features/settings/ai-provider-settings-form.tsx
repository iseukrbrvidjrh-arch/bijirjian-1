import { useCallback, useEffect, useMemo, useState } from "react";
import { CheckCircle2, KeyRound, RefreshCw } from "lucide-react";
import { useForm } from "react-hook-form";

import { Button } from "@/components/ui/button";
import {
  useAiProviderSettings,
  useListAiProviderModels,
  useSaveAiProviderSettings,
  useTestAiProviderConnection,
} from "@/features/settings/ai-provider-queries";
import {
  formatUiError,
  providerModelLabel,
  providerTypeLabel,
} from "@/lib/display";
import type {
  AiProviderModel,
  AiProviderType,
  ProviderModelInfoDto,
  SaveAiProviderSettingsInput,
} from "@/types/ai-provider";
import {
  builtinProviderModels,
  providerDefaultModels,
} from "@/types/ai-provider";

interface ProviderFormValues {
  providerType: AiProviderType;
  defaultModel: AiProviderModel;
  apiKey: string;
}

const providerApiKeyPlaceholders: Record<AiProviderType, string> = {
  deepseek: "请输入 DeepSeek API Key",
  qwen: "请输入 Qwen / DashScope API Key",
  openai: "请输入 OpenAI API Key",
  gemini: "请输入 Gemini API Key",
};

function ensureModelInList(
  models: readonly ProviderModelInfoDto[],
  modelId: string,
  providerType: AiProviderType,
): ProviderModelInfoDto[] {
  if (models.some((model) => model.id === modelId)) {
    return [...models];
  }

  return [
    {
      id: modelId,
      label: providerModelLabel(modelId, models),
      providerType,
      source: "remote",
    },
    ...models,
  ];
}

export function AiProviderSettingsForm() {
  const settingsQuery = useAiProviderSettings();
  const saveMutation = useSaveAiProviderSettings();
  const testMutation = useTestAiProviderConnection();
  const listModelsMutation = useListAiProviderModels();
  const [saveMessage, setSaveMessage] = useState<string>();
  const [saveError, setSaveError] = useState<string>();
  const [modelsMessage, setModelsMessage] = useState<string>();
  const [modelsError, setModelsError] = useState<string>();
  const [modelListsByProvider, setModelListsByProvider] = useState<
    Partial<Record<AiProviderType, ProviderModelInfoDto[]>>
  >({});
  const {
    register,
    handleSubmit,
    resetField,
    setValue,
    watch,
    formState: { errors },
  } = useForm<ProviderFormValues>({
    defaultValues: {
      providerType: "deepseek",
      defaultModel: "deepseek-v4-flash",
      apiKey: "",
    },
  });

  const selectedProviderType = watch("providerType");
  const selectedDefaultModel = watch("defaultModel");
  const apiKeyValue = watch("apiKey");

  const availableModels = useMemo(() => {
    const cached = modelListsByProvider[selectedProviderType];
    const base = cached ?? builtinProviderModels[selectedProviderType];
    return ensureModelInList(base, selectedDefaultModel, selectedProviderType);
  }, [
    modelListsByProvider,
    selectedDefaultModel,
    selectedProviderType,
  ]);

  useEffect(() => {
    if (settingsQuery.data) {
      setValue("providerType", settingsQuery.data.providerType);
      setValue("defaultModel", settingsQuery.data.defaultModel);
    }
  }, [setValue, settingsQuery.data]);

  useEffect(() => {
    if (
      !availableModels.some((model) => model.id === selectedDefaultModel)
    ) {
      setValue("defaultModel", providerDefaultModels[selectedProviderType]);
    }
  }, [
    availableModels,
    selectedDefaultModel,
    selectedProviderType,
    setValue,
  ]);

  useEffect(() => {
    testMutation.reset();
    listModelsMutation.reset();
    setModelsMessage(undefined);
    setModelsError(undefined);
  }, [selectedProviderType]);

  const refreshModels = useCallback(async () => {
    setModelsMessage(undefined);
    setModelsError(undefined);

    try {
      const result = await listModelsMutation.mutateAsync({
        providerType: selectedProviderType,
        apiKey: apiKeyValue.trim() || undefined,
      });

      setModelListsByProvider((current) => ({
        ...current,
        [selectedProviderType]: result.models,
      }));

      if (result.usedFallback) {
        setModelsError(
          result.fallbackReason ??
            "无法获取远程模型列表，已使用内置备用模型列表。",
        );
      } else {
        setModelsMessage("已刷新远程模型列表。");
      }
    } catch (error) {
      setModelListsByProvider((current) => ({
        ...current,
        [selectedProviderType]: builtinProviderModels[selectedProviderType],
      }));
      setModelsError(
        formatUiError(error, "刷新模型列表失败，已使用内置备用模型列表。"),
      );
    }
  }, [apiKeyValue, listModelsMutation, selectedProviderType]);

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
  const savedProviderType = settings?.providerType;
  const hasApiKeyInput = apiKeyValue.trim().length > 0;
  const hasSavedApiKeyForSelection =
    savedProviderType === selectedProviderType && (settings?.hasApiKey ?? false);
  const canRefreshModels =
    hasApiKeyInput ||
    hasSavedApiKeyForSelection ||
    savedProviderType !== selectedProviderType;
  const canTestConnection =
    savedProviderType === selectedProviderType && (settings?.hasApiKey ?? false);

  return (
    <section className="rounded-lg border bg-background p-5">
      <div className="flex items-start gap-3">
        <div className="rounded-md bg-muted p-2">
          <KeyRound className="size-4" />
        </div>
        <div>
          <h2 className="font-semibold">AI 服务</h2>
          <p className="mt-1 text-sm text-muted-foreground">
            配置用于生成总结的 AI 服务商。API Key 按服务商分别保存在 macOS
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
            <option value="qwen">Qwen / 通义千问</option>
            <option value="openai">OpenAI / GPT</option>
            <option value="gemini">Gemini / Google</option>
          </select>
        </div>

        <div>
          <div className="flex items-center justify-between gap-3">
            <label className="text-sm font-medium" htmlFor="default-model">
              默认模型
            </label>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="h-8 px-2"
              disabled={
                !canRefreshModels ||
                saveMutation.isPending ||
                listModelsMutation.isPending
              }
              onClick={() => void refreshModels()}
            >
              <RefreshCw
                className={
                  listModelsMutation.isPending
                    ? "size-3.5 animate-spin"
                    : "size-3.5"
                }
              />
              {listModelsMutation.isPending
                ? "正在刷新…"
                : "刷新模型列表"}
            </Button>
          </div>
          <select
            id="default-model"
            className="mt-2 h-9 w-full rounded-md border bg-background px-3 text-sm outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
            {...register("defaultModel")}
          >
            {availableModels.map((model) => (
              <option key={model.id} value={model.id}>
                {model.label}
                {model.source === "remote" ? "" : "（内置）"}
              </option>
            ))}
          </select>
          <p className="mt-1 text-xs text-muted-foreground">
            生成 AI 总结时默认使用这个模型。优先显示已刷新的远程模型，失败时回退到内置列表。
          </p>
          {modelsMessage && (
            <p className="mt-1 text-xs text-emerald-700" role="status">
              {modelsMessage}
            </p>
          )}
          {modelsError && (
            <p className="mt-1 text-xs text-amber-700" role="alert">
              {modelsError}
            </p>
          )}
          {!canRefreshModels && (
            <p className="mt-1 text-xs text-muted-foreground">
              填写 API Key 或保存该服务商配置后，可刷新最新模型列表；若钥匙串中已有该服务商
              Key，也可直接刷新。
            </p>
          )}
        </div>

        <div>
          <div className="flex items-center justify-between gap-3">
            <label className="text-sm font-medium" htmlFor="provider-api-key">
              API Key
            </label>
            <span className="text-xs text-muted-foreground">
              {hasSavedApiKeyForSelection
                ? "已保存到钥匙串"
                : savedProviderType !== selectedProviderType
                  ? "切换服务商后请先保存"
                  : "尚未配置"}
            </span>
          </div>
          <input
            id="provider-api-key"
            type="password"
            autoComplete="new-password"
            className="mt-2 h-9 w-full rounded-md border bg-background px-3 text-sm outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
            placeholder={
              hasSavedApiKeyForSelection
                ? "留空可继续使用已保存的 API Key"
                : providerApiKeyPlaceholders[selectedProviderType]
            }
            {...register("apiKey", {
              validate: (value, formValues) => {
                if (value.trim().length > 0) {
                  return true;
                }
                if (
                  settings?.providerType === formValues.providerType &&
                  (settings?.hasApiKey ?? false)
                ) {
                  return true;
                }
                if (settings?.providerType !== formValues.providerType) {
                  return true;
                }
                return "首次保存该服务商时必须填写 API Key。";
              },
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
                {providerTypeLabel(testMutation.data.providerType)} 连接成功。
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
            {!canTestConnection && !testMutation.isPending && (
              <span className="text-muted-foreground">
                请先保存当前服务商的 API Key，再测试连接。
              </span>
            )}
          </div>

          <div className="flex items-center gap-2">
            <Button
              type="button"
              variant="outline"
              disabled={
                !canTestConnection ||
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
