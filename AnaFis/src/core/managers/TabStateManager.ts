import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import type { Tab } from '@/core/types/tabs';

interface TabState {
  tabs: Tab[];
  activeTabId: string | null;
  
  // Actions
  addTab: (tab: Tab) => void;
  removeTab: (id: string) => void;
  renameTab: (id: string, newTitle: string) => void;
  setActiveTab: (id: string) => void;
  reorderTabs: (sourceIndex: number, targetIndex: number) => void;
  initializeTabs: (initialTabs: Tab[]) => void;
  updateTabState: (id: string, state: Record<string, unknown>) => void;
}

export const useTabStore = create<TabState>()(
  subscribeWithSelector((set, get) => ({
    tabs: [],
    activeTabId: null,

    addTab: (tab: Tab) => set((state) => {
      // Check if tab with this ID already exists
      if (state.tabs.some((existingTab) => existingTab.id === tab.id)) {
        return { ...state, activeTabId: tab.id };
      }
      
      // Memory management: Warn if too many tabs are open
      const totalTabs = state.tabs.length + 1;
      if (totalTabs > 10) {
        console.warn(`Warning: ${totalTabs} tabs are open. Consider closing unused tabs to improve performance.`);
      }
      
      return {
        tabs: [...state.tabs, tab],
        activeTabId: tab.id
      };
    }),

    removeTab: (id: string) => {
      const state = get();
      const newTabs = state.tabs.filter((t) => t.id !== id);
      const newActiveId = state.activeTabId === id
        ? newTabs[0]?.id ?? null
        : state.activeTabId;

      set({ tabs: newTabs, activeTabId: newActiveId });
    },

    renameTab: (id: string, newTitle: string) => set((state) => ({
      tabs: state.tabs.map((tab) =>
        tab.id === id ? { ...tab, title: newTitle } : tab
      ),
    })),

    setActiveTab: (id: string) => set({ activeTabId: id }),

    reorderTabs: (sourceIndex: number, targetIndex: number) => set((state) => {
      const newTabs = [...state.tabs];
      const [removed] = newTabs.splice(sourceIndex, 1);

      if (!removed) {
        console.warn('Failed to reorder tabs: source index out of bounds');
        return state;
      }

      newTabs.splice(targetIndex, 0, removed);
      return { tabs: newTabs };
    }),

    initializeTabs: (initialTabs: Tab[]) => set({
      tabs: initialTabs,
      activeTabId: initialTabs[0]?.id ?? null
    }),

    updateTabState: (id: string, state: Record<string, unknown>) => {
      // Update frontend state
      set((currentState) => ({
        tabs: currentState.tabs.map((tab) =>
          tab.id === id ? { ...tab, state: { ...tab.state, ...state } } : tab
        ),
      }));
    }
  }))
);