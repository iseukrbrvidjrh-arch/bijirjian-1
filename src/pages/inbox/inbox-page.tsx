import { CaptureForm } from "@/features/capture/capture-form";
import { InboxSourceList } from "@/pages/inbox/inbox-source-list";

export function InboxPage() {
  return (
    <section className="mx-auto max-w-3xl">
      <div>
        <h1 className="text-2xl font-semibold">Inbox</h1>
        <p className="mt-2 text-sm text-muted-foreground">
          Capture text locally and keep it ready for later processing.
        </p>
      </div>

      <div className="mt-6 space-y-6">
        <CaptureForm />
        <InboxSourceList />
      </div>
    </section>
  );
}
