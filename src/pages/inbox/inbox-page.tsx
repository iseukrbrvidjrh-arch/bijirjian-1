import { CaptureForm } from "@/features/capture/capture-form";
import { PdfCaptureForm } from "@/features/capture/pdf-capture-form";
import { InboxSourceList } from "@/pages/inbox/inbox-source-list";

export function InboxPage() {
  return (
    <section className="mx-auto max-w-3xl">
      <div>
        <h1 className="text-2xl font-semibold">收集箱</h1>
        <p className="mt-2 text-sm text-muted-foreground">
          随手记录文字或导入含文字的 PDF，稍后再整理成知识。
        </p>
      </div>

      <div className="mt-6 space-y-6">
        <CaptureForm />
        <PdfCaptureForm />
        <InboxSourceList />
      </div>
    </section>
  );
}
