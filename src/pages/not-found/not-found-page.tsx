import { Link } from "react-router-dom";

import { Button } from "@/components/ui/button";

export function NotFoundPage() {
  return (
    <section className="mx-auto max-w-2xl rounded-lg border bg-background p-6">
      <p className="text-sm font-medium text-muted-foreground">404</p>
      <h1 className="mt-2 text-2xl font-semibold">页面不存在</h1>
      <p className="mt-2 text-sm text-muted-foreground">
        你访问的页面不存在，可能已经移动或地址有误。
      </p>
      <div className="mt-5 flex flex-wrap gap-2">
        <Button asChild>
          <Link to="/dashboard">返回总览</Link>
        </Button>
        <Button asChild variant="outline">
          <Link to="/inbox">前往收集箱</Link>
        </Button>
      </div>
    </section>
  );
}
