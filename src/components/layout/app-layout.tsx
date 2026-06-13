import { Brain, Inbox, Library, Menu, Settings } from "lucide-react";
import { NavLink, Outlet } from "react-router-dom";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { useAppShellStore } from "@/stores/app-shell-store";

const navigation = [
  { to: "/inbox", label: "Inbox", icon: Inbox },
  { to: "/knowledge", label: "Knowledge", icon: Library },
  { to: "/settings", label: "Settings", icon: Settings },
];

export function AppLayout() {
  const isSidebarOpen = useAppShellStore((state) => state.isSidebarOpen);
  const toggleSidebar = useAppShellStore((state) => state.toggleSidebar);

  return (
    <div className="flex min-h-screen bg-muted/30">
      {isSidebarOpen && (
        <aside className="w-64 border-r bg-background p-4">
          <div className="mb-8 flex items-center gap-2 px-2">
            <Brain className="size-5" />
            <span className="font-semibold">Second Brain OS</span>
          </div>

          <nav className="space-y-1" aria-label="Primary navigation">
            {navigation.map(({ to, label, icon: Icon }) => (
              <NavLink
                key={to}
                to={to}
                className={({ isActive }) =>
                  cn(
                    "flex items-center gap-3 rounded-md px-3 py-2 text-sm text-muted-foreground",
                    isActive && "bg-accent text-accent-foreground",
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
        <header className="flex h-14 items-center border-b bg-background px-4">
          <Button
            variant="ghost"
            size="icon"
            onClick={toggleSidebar}
            aria-label="Toggle navigation"
          >
            <Menu />
          </Button>
        </header>

        <main className="p-6">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
