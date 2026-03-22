// Optimized App.tsx

import {
  DndContext,
  type DragEndEvent,
  DragOverlay,
  type DragStartEvent,
  PointerSensor,
  rectIntersection,
  useSensor,
  useSensors,
} from '@dnd-kit/core';
import { Box } from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import React, {
  lazy,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';
import { NotificationProvider } from '@/core/contexts/NotificationContext';
// Contexts
import { WorkbookDataProvider } from '@/core/contexts/WorkbookDataContext';
import { useNotification } from '@/core/managers/NotificationManager';
// Managers (State Management)
import { useTabStore } from '@/core/managers/TabStateManager';
import { useWorkbookData } from '@/core/managers/WorkbookDataProvider';
import type { WorkbookData } from '@/core/types/import';
// Types
import type { Tab } from '@/core/types/tabs';
import AppToolbar from '@/shared/components/AppToolbar';
import CustomTitleBar from '@/shared/components/CustomTitleBar';
// Components
import { DraggableTabBar } from '@/shared/components/DraggableTabBar';
import DragOverlayComponent from '@/shared/components/DragOverlayComponent';
import OptimizedTabRenderer from '@/shared/components/OptimizedTabRenderer';
import { anafisTheme } from '@/shared/theme/unifiedTheme';

// Lazy load tab components for code splitting
const HomeTab = lazy(() => import('@/tabs/home/HomeTab'));
const SpreadsheetTab = lazy(() => import('@/tabs/spreadsheet/SpreadsheetTab'));
const FittingTab = lazy(() => import('@/tabs/fitting/FittingTab'));
const SolverTab = lazy(() => import('@/tabs/solver/SolverTab'));

// Static styles to prevent recreation
const MAIN_CONTAINER_STYLES = {
  display: 'flex',
  flexDirection: 'column',
  height: '100vh',
  width: '100vw',
  margin: 0,
  padding: 0,
  overflow: 'hidden',
  backgroundColor: anafisTheme.colors.background.primary,
  position: 'relative',
} as const;

const CONTENT_AREA_STYLES = {
  display: 'flex',
  flexDirection: 'column',
  flexGrow: 1,
  width: '100%',
  overflow: 'hidden',
  margin: 0,
  padding: 0,
} as const;

const TAB_CONTENT_STYLES = {
  flexGrow: 1,
  p: 0,
  bgcolor: anafisTheme.colors.background.primary,
  overflow: 'auto',
  width: '100%',
  margin: 0,
} as const;

function App() {
  // Store and state
  const { tabs, activeTabId, addTab: storeAddTab, reorderTabs } = useTabStore();

  // Contexts
  const { setPendingWorkbookData } = useWorkbookData();
  const { showNotification } = useNotification(); // Drag state
  const [draggedTab, setDraggedTab] = useState<Tab | null>(null);

  // Ref to store handleAddTab function
  const handleAddTabRef = useRef<
    | ((
        id: string,
        title: string,
        content: React.ReactNode,
        workbookData?: WorkbookData
      ) => void)
    | null
  >(null);

  // Memoized tab content factory
  const createTabContent = useCallback((tabType: string, tabId: string) => {
    const safeOpenNewTab = handleAddTabRef.current ?? (() => {});
    switch (tabType) {
      case 'home':
        return <HomeTab openNewTab={safeOpenNewTab} />;
      case 'spreadsheet':
        return <SpreadsheetTab tabId={tabId} />;
      case 'fitting':
        return <FittingTab />;
      case 'solver':
        return <SolverTab />;
      default:
        return <HomeTab openNewTab={safeOpenNewTab} />;
    }
  }, []);

  // Memoized add tab handler
  const handleAddTab = useCallback(
    (
      id: string,
      title: string,
      content: React.ReactNode,
      workbookData?: WorkbookData
    ) => {
      const tabType = id.split('-')[0] ?? 'home';
      const newTab: Tab = {
        id,
        title,
        content: content ?? createTabContent(tabType, id),
        type: tabType,
      };
      storeAddTab(newTab);

      // Store workbook data for later loading if provided
      if (workbookData) {
        setPendingWorkbookData(id, workbookData);
      }
    },
    [storeAddTab, createTabContent, setPendingWorkbookData]
  );

  // Update the ref after handleAddTab is defined
  useEffect(() => {
    handleAddTabRef.current = handleAddTab;
  }, [handleAddTab]);

  // App initialization
  useEffect(() => {
    // Initialize with home tab if no tabs exist
    if (tabs.length === 0) {
      const homeTab: Tab = {
        id: 'home',
        title: 'Home',
        content: createTabContent('home', 'home'),
        type: 'home',
      };
      storeAddTab(homeTab);
    }
  }, [tabs.length, storeAddTab, createTabContent]);

  // Check for file associations on mount
  useEffect(() => {
    let isMounted = true;

    // We only want to process this once on mount
    const checkStartupFile = async () => {
      try {
        const filePath = await invoke<string | null>('get_startup_file');
        if (filePath && isMounted) {
          console.log('Startup file detected:', filePath);

          // Import the .anafispread file
          const workbookData = await invoke<WorkbookData>(
            'import_anafis_spread_direct',
            { filePath }
          );

          const fileName =
            filePath
              .split('/')
              .pop()
              ?.replace(/\.anafispread|\.anafis/i, '') ?? 'Opened File';
          const tabId = `spreadsheet-opened-${Date.now()}`;

          // Instead of immediate adding via ref which could be missing, schedule it
          // if handleAddTabRef isn't perfectly registered yet
          if (handleAddTabRef.current) {
            handleAddTabRef.current(tabId, fileName, undefined, workbookData);
          } else {
            // Give the app a moment to finish mounting the callback ref
            setTimeout(() => {
              if (handleAddTabRef.current) {
                handleAddTabRef.current(
                  tabId,
                  fileName,
                  undefined,
                  workbookData
                );
              }
            }, 100);
          }
        }
      } catch (error) {
        if (!isMounted) return;
        console.error('Failed to open startup file:', error);
        const errorMessage =
          error instanceof Error ? error.message : 'Unknown error occurred';
        showNotification({
          type: 'error',
          message: `Failed to open file: ${errorMessage}`,
        });
      }
    };

    void checkStartupFile();

    return () => {
      isMounted = false;
    };
  }, [showNotification]);

  // Drag and drop setup
  const sensors = useSensors(useSensor(PointerSensor));

  // Memoized drag handlers
  const handleDragStart = useCallback(
    (event: DragStartEvent) => {
      const tab = tabs.find((t: Tab) => t.id === event.active.id);
      setDraggedTab(tab ?? null);
    },
    [tabs]
  );

  const handleDragEnd = useCallback(
    (event: DragEndEvent) => {
      const { active, over } = event;
      setDraggedTab(null);

      if (over && active.id !== over.id) {
        // Handle reordering
        const activeIndex = tabs.findIndex((tab) => tab.id === active.id);
        const overIndex = tabs.findIndex((tab) => tab.id === over.id);
        if (activeIndex !== -1 && overIndex !== -1) {
          reorderTabs(activeIndex, overIndex);
        }
      }
    },
    [tabs, reorderTabs]
  );

  // Memoized content area height calculation
  const contentAreaHeight = useMemo(() => 'calc(100vh - 96px)', []);

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={rectIntersection}
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
    >
      <DragOverlay>
        {draggedTab && <DragOverlayComponent draggedTab={draggedTab} />}
      </DragOverlay>

      <Box
        sx={MAIN_CONTAINER_STYLES}
        onContextMenu={(e: React.MouseEvent) => {
          e.preventDefault();
          return false;
        }}
      >
        {/* Custom Title Bar */}
        <CustomTitleBar title="AnaFis" />

        {/* Toolbar */}
        <AppToolbar
          onAddTab={handleAddTab}
          createTabContent={createTabContent}
        />

        {/* Main Content Area */}
        <Box sx={{ ...CONTENT_AREA_STYLES, height: contentAreaHeight }}>
          {/* Tab Bar */}
          <DraggableTabBar />

          {/* Tab Content */}
          <Box sx={TAB_CONTENT_STYLES}>
            <OptimizedTabRenderer tabs={tabs} activeTabId={activeTabId} />
          </Box>
        </Box>
      </Box>
    </DndContext>
  );
}

const AppWithProviders = () => (
  <NotificationProvider>
    <WorkbookDataProvider>
      <App />
    </WorkbookDataProvider>
  </NotificationProvider>
);

export default React.memo(AppWithProviders);
