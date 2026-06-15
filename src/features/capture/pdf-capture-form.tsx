import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";

import { Button } from "@/components/ui/button";
import {
  parsePdfSourceMetadata,
} from "@/features/capture/pdf-source-metadata";
import { useCapturePdfSource } from "@/features/capture/source-queries";
import { formatUiError } from "@/lib/display";

export function PdfCaptureForm() {
  const [isSelecting, setIsSelecting] = useState(false);
  const [selectionError, setSelectionError] = useState<string>();
  const captureMutation = useCapturePdfSource();
  const isPending = isSelecting || captureMutation.isPending;
  const importedMetadata = captureMutation.data
    ? parsePdfSourceMetadata(captureMutation.data.metadataJson)
    : null;

  async function selectAndImportPdf() {
    if (isPending) {
      return;
    }

    setSelectionError(undefined);
    captureMutation.reset();
    setIsSelecting(true);

    try {
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [{ name: "PDF", extensions: ["pdf"] }],
      });

      if (selected === null) {
        return;
      }
      if (Array.isArray(selected)) {
        setSelectionError("请选择一个 PDF 文件。");
        return;
      }

      await captureMutation.mutateAsync(selected);
    } catch (error) {
      if (!captureMutation.isError) {
        setSelectionError(
          formatUiError(error, "PDF 导入失败，请重新选择文件。"),
        );
      }
    } finally {
      setIsSelecting(false);
    }
  }

  return (
    <section className="rounded-lg border bg-background p-4">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <h2 className="text-sm font-medium">导入 PDF</h2>
          <p className="mt-1 text-xs text-muted-foreground">
            从本地 PDF 提取文字并加入收集箱。扫描版 PDF 需要 OCR，
            当前暂不支持。
          </p>
        </div>
        <Button
          type="button"
          variant="outline"
          disabled={isPending}
          onClick={() => void selectAndImportPdf()}
        >
          {isSelecting
            ? "正在选择 PDF…"
            : captureMutation.isPending
              ? "正在导入 PDF…"
              : "选择并导入 PDF"}
        </Button>
      </div>

      <div className="mt-3 min-h-5 text-sm" aria-live="polite">
        {(selectionError || captureMutation.isError) && (
          <p className="text-destructive" role="alert">
            {selectionError ??
              formatUiError(
                captureMutation.error,
                "PDF 导入失败，请确认文件有效且包含可提取文字。",
              )}
          </p>
        )}
        {captureMutation.isSuccess && (
          <p className="text-emerald-700">
            已将“
            {importedMetadata?.originalFileName ?? "PDF 文档"}
            ”加入收集箱。
          </p>
        )}
      </div>
    </section>
  );
}
