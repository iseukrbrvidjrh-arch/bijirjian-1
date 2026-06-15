import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";

import { Button } from "@/components/ui/button";
import {
  parsePdfSourceMetadata,
} from "@/features/capture/pdf-source-metadata";
import { useCapturePdfSource } from "@/features/capture/source-queries";

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
        setSelectionError("Please select a single PDF file.");
        return;
      }

      await captureMutation.mutateAsync(selected);
    } catch (error) {
      if (!captureMutation.isError) {
        setSelectionError(
          error instanceof Error ? error.message : String(error),
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
          <h2 className="text-sm font-medium">Import a PDF</h2>
          <p className="mt-1 text-xs text-muted-foreground">
            Extract text from one local PDF. Scanned documents require OCR
            and are not supported yet.
          </p>
        </div>
        <Button
          type="button"
          variant="outline"
          disabled={isPending}
          onClick={() => void selectAndImportPdf()}
        >
          {isSelecting
            ? "Selecting PDF..."
            : captureMutation.isPending
              ? "Importing PDF..."
              : "Select PDF / Import PDF"}
        </Button>
      </div>

      <div className="mt-3 min-h-5 text-sm" aria-live="polite">
        {(selectionError || captureMutation.isError) && (
          <p className="text-destructive" role="alert">
            {selectionError ?? captureMutation.error?.message}
          </p>
        )}
        {captureMutation.isSuccess && (
          <p>
            Imported{" "}
            {importedMetadata?.originalFileName ?? "PDF source"} into Inbox.
          </p>
        )}
      </div>
    </section>
  );
}
