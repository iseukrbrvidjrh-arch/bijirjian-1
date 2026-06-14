import { useEffect, useState } from "react";
import { CheckCircle2, KeyRound } from "lucide-react";
import { useForm } from "react-hook-form";

import { Button } from "@/components/ui/button";
import {
  useAiProviderSettings,
  useSaveAiProviderSettings,
  useTestAiProviderConnection,
} from "@/features/settings/ai-provider-queries";
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
      setSaveMessage("Provider settings saved securely.");
      saveMutation.reset();
    } catch (error) {
      setSaveError(errorMessage(error));
      saveMutation.reset();
    }
  }

  function testConnection() {
    setSaveMessage(undefined);
    setSaveError(undefined);
    testMutation.mutate();
  }

  if (settingsQuery.isPending) {
    return <SettingsState>Loading AI provider settings...</SettingsState>;
  }

  if (settingsQuery.isError) {
    return (
      <SettingsState tone="error">
        Could not load AI provider settings:{" "}
        {errorMessage(settingsQuery.error)}
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
          <h2 className="font-semibold">AI Provider</h2>
          <p className="mt-1 text-sm text-muted-foreground">
            Configure DeepSeek for future AI workflows. Your API key is
            stored in the macOS Keychain and is never displayed again.
          </p>
        </div>
      </div>

      <form
        className="mt-6 space-y-5"
        onSubmit={handleSubmit(saveSettings)}
      >
        <div>
          <label className="text-sm font-medium" htmlFor="provider-type">
            Provider
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
            Default model
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
            Used as the default model for future AI workflows.
          </p>
        </div>

        <div>
          <div className="flex items-center justify-between gap-3">
            <label className="text-sm font-medium" htmlFor="provider-api-key">
              API Key
            </label>
            <span className="text-xs text-muted-foreground">
              {hasSavedApiKey ? "Saved in Keychain" : "Not configured"}
            </span>
          </div>
          <input
            id="provider-api-key"
            type="password"
            autoComplete="new-password"
            className="mt-2 h-9 w-full rounded-md border bg-background px-3 text-sm outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
            placeholder={
              hasSavedApiKey
                ? "Leave blank to keep the saved API key"
                : "Enter your DeepSeek API key"
            }
            {...register("apiKey", {
              validate: (value) =>
                hasSavedApiKey ||
                value.trim().length > 0 ||
                "API key is required for the first save.",
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
              <span className="inline-flex items-center gap-1.5 text-foreground">
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
              <span className="inline-flex items-center gap-1.5 text-foreground">
                <CheckCircle2 className="size-4" />
                {testMutation.data.message}
              </span>
            )}
            {testMutation.isError && (
              <span className="text-destructive" role="alert">
                {errorMessage(testMutation.error)}
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
                ? "Testing..."
                : "Test connection"}
            </Button>
            <Button
              type="submit"
              disabled={
                saveMutation.isPending || testMutation.isPending
              }
            >
              {saveMutation.isPending ? "Saving..." : "Save settings"}
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

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
