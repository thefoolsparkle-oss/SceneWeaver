import { create } from 'zustand';

interface UiState {
  selectedLibraryId: string | null;
  setSelectedLibraryId: (id: string | null) => void;
  sidebarOpen: boolean;
  toggleSidebar: () => void;
}

export const useUiStore = create<UiState>((set) => ({
  selectedLibraryId: null,
  setSelectedLibraryId: (id) => set({ selectedLibraryId: id }),
  sidebarOpen: true,
  toggleSidebar: () => set((state) => ({ sidebarOpen: !state.sidebarOpen })),
}));
