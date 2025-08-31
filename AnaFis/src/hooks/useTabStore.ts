import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import type { Tab } from '../types/tabs';
import { invoke } from '@tauri-apps/api/core';

interface TabState {
  tabs: Tab[];
  activeTabId: string | null;
  draggedTab: Tab | null;
  addTab: (tab: Tab) => void;
  removeTab: (id: string) => Promise<void>;
  renameTab: (id: string, newTitle: string) => void;
  setActiveTab: (id: string) => void;
  reorderTabs: (sourceIndex: number, targetIndex: number) => void;
  setDraggedTab: (tab: Tab | null) => void;
  detachTab: (id: string, position?: { x: number; y: number }) => Promise<void>;
  initializeTabs: (initialTabs: Tab[]) => void;
}

export const useTabStore = create<TabState>()(
  subscribeWithSelector((set, get) => ({
    tabs: [],
    activeTabId: null,
    draggedTab: null,

    addTab: (tab: Tab) => set((state) => {
      // Check if tab with this ID already exists
      if (state.tabs.some((existingTab) => existingTab.id === tab.id)) {
        // If it exists, just set it as active
        return {
          ...state,
          activeTabId: tab.id
        };
      }
      // Otherwise add the new tab and make it active
      return {
        tabs: [...state.tabs, tab],
        activeTabId: tab.id
      };
    }),

    removeTab: async (id: string) => {
      const state = get();
      const newTabs = state.tabs.filter((t) => t.id !== id);
      const newActiveId = state.activeTabId === id
        ? newTabs[0]?.id || null
        : state.activeTabId;

      console.log('removeTab called:', { id, newTabsLength: newTabs.length, currentTabs: state.tabs.length });

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
      newTabs.splice(targetIndex, 0, removed);
      return { tabs: newTabs };
    }),

    setDraggedTab: (tab: Tab | null) => set({ draggedTab: tab }),

    detachTab: async (id: string, position?: { x: number; y: number }) => {
      const { tabs } = get();
      const tabToDetach = tabs.find((t) => t.id === id);
      if (!tabToDetach || id === 'home') return;

      // Remove from current store
      await get().removeTab(id);

      try {
        // Create detached window via Tauri with position
        await invoke('create_tab_window', {
          tabInfo: {
            id: tabToDetach.id,
            title: tabToDetach.title,
            content_type: id.split('-')[0],
            state: {},
            icon: null
          },
          position: position
        });
      } catch (error) {
        console.error('Failed to detach tab:', error);
        // Restore tab on failure
        get().addTab(tabToDetach);
      }
    },

    initializeTabs: (initialTabs: Tab[]) => set({
      tabs: initialTabs,
      activeTabId: initialTabs[0]?.id || null
    })
  }))
);
