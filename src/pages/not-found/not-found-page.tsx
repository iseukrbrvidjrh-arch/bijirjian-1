import { Link } from "react-router-dom";

import { Button } from "@/components/ui/button";

export function NotFoundPage() {
  return (
    <section className="mx-auto max-w-2xl rounded-lg border bg-background p-6">
      <p className="text-sm font-medium text-muted-foreground">404</p>
      <h1 className="mt-2 text-2xl font-semibold">Page not found</h1>
      <p className="mt-2 text-sm text-muted-foreground">
        The requested page does not exist in Second Brain OS.
      </p>
      <div className="mt-5 flex flex-wrap gap-2">
        <Button asChild>
          <Link to="/dashboard">Go to Dashboard</Link>
        </Button>
        <Button asChild variant="outline">
          <Link to="/inbox">Go to Inbox</Link>
        </Button>
      </div>
    </section>
  );
}
