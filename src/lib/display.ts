import type { AiProviderModel, AiProviderType } from "@/types/ai-provider";
import type { ExportStatus } from "@/types/export";
import type {
  KnowledgeStatus,
  KnowledgeType,
} from "@/types/knowledge";
import type { InboxStatus, SourceType } from "@/types/source";
import type { AiRunStatus } from "@/types/summary";

const sourceTypeLabels: Record<SourceType, string> = {
  text: "文本",
  pdf: "PDF 文档",
};

const inboxStatusLabels: Record<InboxStatus, string> = {
  unprocessed: "未处理",
  processed: "已处理",
  dismissed: "已忽略",
  failed: "处理失败",
};

const knowledgeStatusLabels: Record<KnowledgeStatus, string> = {
  proposed: "待审核",
  accepted: "已收录",
  archived: "已归档",
};

const knowledgeTypeLabels: Record<KnowledgeType, string> = {
  concept: "概念",
  tool: "工具",
  project: "项目",
  question: "问题",
  solution: "解决方案",
  insight: "洞察",
  resource: "资料",
  person: "人物",
};

const aiRunStatusLabels: Record<AiRunStatus, string> = {
  succeeded: "成功",
  failed: "失败",
};

const exportStatusLabels: Record<ExportStatus, string> = {
  succeeded: "成功",
  failed: "失败",
};

const providerTypeLabels: Record<AiProviderType, string> = {
  deepseek: "DeepSeek",
};

const providerModelLabels: Record<AiProviderModel, string> = {
  "deepseek-v4-flash": "DeepSeek V4 Flash",
  "deepseek-v4-pro": "DeepSeek V4 Pro",
};

export function sourceTypeLabel(value: SourceType) {
  return sourceTypeLabels[value];
}

export function inboxStatusLabel(value: InboxStatus) {
  return inboxStatusLabels[value];
}

export function knowledgeStatusLabel(value: KnowledgeStatus) {
  return knowledgeStatusLabels[value];
}

export function knowledgeTypeLabel(value: KnowledgeType) {
  return knowledgeTypeLabels[value];
}

export function aiRunStatusLabel(value: AiRunStatus) {
  return aiRunStatusLabels[value];
}

export function exportStatusLabel(value: ExportStatus) {
  return exportStatusLabels[value];
}

export function providerTypeLabel(value: AiProviderType) {
  return providerTypeLabels[value];
}

export function providerModelLabel(value: AiProviderModel) {
  return providerModelLabels[value];
}

export function formatDateTime(value: string) {
  const date = new Date(value);

  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return new Intl.DateTimeFormat("zh-CN", {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(date);
}

export function formatUiError(
  error: unknown,
  fallback = "操作失败，请稍后重试。",
) {
  const message = error instanceof Error ? error.message : String(error ?? "");

  if (!message) {
    return fallback;
  }

  if (/[\u3400-\u9fff]/u.test(message)) {
    return message;
  }

  const normalized = message.toLowerCase();

  if (
    normalized.includes("401") ||
    normalized.includes("403") ||
    normalized.includes("authentication") ||
    normalized.includes("unauthorized") ||
    normalized.includes("forbidden")
  ) {
    return "身份验证失败，请检查 DeepSeek API Key。";
  }

  if (normalized.includes("429") || normalized.includes("rate limit")) {
    return "请求过于频繁，请稍后再试。";
  }

  if (normalized.includes("timeout") || normalized.includes("timed out")) {
    return "请求超时，请检查网络后重试。";
  }

  if (
    normalized.includes("network") ||
    normalized.includes("connection") ||
    normalized.includes("connect")
  ) {
    return "网络连接失败，请检查网络后重试。";
  }

  if (normalized.includes("api key")) {
    return "尚未配置有效的 DeepSeek API Key。";
  }

  if (
    normalized.includes("provider") &&
    normalized.includes("not configured")
  ) {
    return "尚未配置 AI 服务，请先前往设置。";
  }

  if (normalized.includes("not found")) {
    return "没有找到对应内容，它可能已被删除或不在当前工作区。";
  }

  if (
    normalized.includes("conflict") ||
    normalized.includes("already") ||
    normalized.includes("cannot transition")
  ) {
    return "当前状态不允许执行此操作，或对应记录已经存在。";
  }

  if (
    normalized.includes("vault") &&
    (normalized.includes("directory") ||
      normalized.includes("path") ||
      normalized.includes("exist"))
  ) {
    return "Obsidian 仓库路径无效，请确认路径存在且为文件夹。";
  }

  if (
    normalized.includes("pdf") &&
    (normalized.includes("extract") ||
      normalized.includes("encrypted") ||
      normalized.includes("parse"))
  ) {
    return "无法读取这个 PDF。请确认文件未加密且包含可提取的文字。";
  }

  if (
    normalized.includes("ocr") ||
    normalized.includes("no extractable text")
  ) {
    return "这个 PDF 没有可提取的文字，当前版本暂不支持 OCR。";
  }

  return fallback;
}
