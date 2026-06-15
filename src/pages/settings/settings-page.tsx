import { AiProviderSettingsForm } from "@/features/settings/ai-provider-settings-form";
import { ObsidianSettingsForm } from "@/features/settings/obsidian-settings-form";
import { PromptSettingsForm } from "@/features/settings/prompt-settings-form";

export function SettingsPage() {
  return (
    <section className="mx-auto max-w-3xl">
      <div>
        <h1 className="text-2xl font-semibold">设置</h1>
        <p className="mt-2 text-sm text-muted-foreground">
          管理 AI 服务、Obsidian 仓库和总结提示词。
        </p>
      </div>

      <div className="mt-6 space-y-6">
        <AiProviderSettingsForm />
        <ObsidianSettingsForm />
        <PromptSettingsForm />
      </div>
    </section>
  );
}
