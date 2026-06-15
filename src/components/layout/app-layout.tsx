import {
  Brain,
  Inbox,
  LayoutDashboard,
  Library,
  Menu,
  Settings,
} from "lucide-react";
import { NavLink, Outlet } from "react-router-dom";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { useAppShellStore } from "@/stores/app-shell-store";

const navigation = [
  { to: "/dashboard", label: "总览", icon: LayoutDashboard },
  { to: "/inbox", label: "收集箱", icon: Inbox },
  { to: "/knowledge", label: "知识库", icon: Library },
  { to: "/settings", label: "设置", icon: Settings },
];

export function AppLayout() {
  const isSidebarOpen = useAppShellStore((state) => state.isSidebarOpen);
  const toggleSidebar = useAppShellStore((state) => state.toggleSidebar);

  return (
    <div className="flex min-h-screen bg-muted/40">
      {isSidebarOpen && (
        <aside className="w-64 border-r border-sidebar-border bg-sidebar p-4 text-sidebar-foreground">
          <div className="mb-8 flex items-center gap-2 px-2">
            <div className="rounded-lg bg-primary p-2 text-primary-foreground shadow-sm">
              <Brain className="size-4" />
            </div>
            <div>
              <span className="block font-semibold">Second Brain OS</span>
              <span className="text-xs text-muted-foreground">
                本地知识助手
              </span>
            </div>
          </div>

          <nav className="space-y-1" aria-label="主导航">
            {navigation.map(({ to, label, icon: Icon }) => (
              <NavLink
                key={to}
                to={to}
                className={({ isActive }) =>
                  cn(
                    "flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm text-muted-foreground transition-colors hover:bg-sidebar-accent/70 hover:text-sidebar-accent-foreground",
                    isActive &&
                      "bg-sidebar-accent font-medium text-sidebar-accent-foreground shadow-sm",
                  )
                }
              >
                <Icon className="size-4" />
                {label}
              </NavLink>
            ))}
          </nav>
        </aside>
      )}

      <div className="min-w-0 flex-1">
        <header className="flex h-14 items-center border-b bg-background/90 px-4 backdrop-blur">
          <Button
            variant="ghost"
            size="icon"
            onClick={toggleSidebar}
            aria-label="展开或收起导航"
          >
            <Menu />
          </Button>
        </header>

        <main className="p-6 lg:p-8">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
