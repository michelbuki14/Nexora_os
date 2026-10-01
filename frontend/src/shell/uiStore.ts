import { create } from "zustand";

// UI-only state (sidebar collapse, command palette). Server and session state
// live elsewhere — this store never holds data from the API.

interface UiState {
  sidebarCollapsed: boolean;
  mobileOpen: boolean;
  commandPaletteOpen: boolean;
  toggleSidebar: () => void;
  setMobileOpen: (open: boolean) => void;
  setCommandPaletteOpen: (open: boolean) => void;
}

export const useUiStore = create<UiState>((set) => ({
  sidebarCollapsed: false,
  mobileOpen: false,
  commandPaletteOpen: false,
  toggleSidebar: () => set((s) => ({ sidebarCollapsed: !s.sidebarCollapsed })),
  setMobileOpen: (open) => set({ mobileOpen: open }),
  setCommandPaletteOpen: (open) => set({ commandPaletteOpen: open }),
}));
