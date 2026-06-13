import { create } from "zustand";

interface AppShellState {
  isSidebarOpen: boolean;
  toggleSidebar: () => void;
}

export const useAppShellStore = create<AppShellState>((set) => ({
  isSidebarOpen: true,
  toggleSidebar: () =>
    set((state) => ({ isSidebarOpen: !state.isSidebarOpen })),
}));
