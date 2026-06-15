import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

type StatusTone =
  | "blue"
  | "green"
  | "gray"
  | "amber"
  | "red"
  | "violet";

const toneClasses: Record<StatusTone, string> = {
  blue: "border-blue-200 bg-blue-50 text-blue-700",
  green: "border-emerald-200 bg-emerald-50 text-emerald-700",
  gray: "border-slate-200 bg-slate-100 text-slate-600",
  amber: "border-amber-200 bg-amber-50 text-amber-700",
  red: "border-red-200 bg-red-50 text-red-700",
  violet: "border-violet-200 bg-violet-50 text-violet-700",
};

export function StatusBadge({
  children,
  tone = "gray",
  className,
}: {
  children: ReactNode;
  tone?: StatusTone;
  className?: string;
}) {
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-full border px-2.5 py-1 text-xs font-medium",
        toneClasses[tone],
        className,
      )}
    >
      {children}
    </span>
  );
}
