// Optimized App.tsx 
import React, { useCallback, lazy, useMemo, useRef, useEffect, useState } from 'react';
import { Box } from '@mui/material';
import {
  DndContext,
  DragEndEvent,
  DragStartEvent,
  DragOverlay,
  PointerSensor,
  useSensor,
  useSensors,
  rectIntersection,
} from '@dnd-kit/core';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

// Components
import { DraggableTabBar } from './components/DraggableTabBar';
import CustomTitleBar from './components/CustomTitleBar';
import AppToolbar from './components/AppToolbar';
import DragOverlayComponent from './components/DragOverlayComponent';
import OptimizedTabRenderer from './components/OptimizedTabRenderer';

// Contexts
import { WorkbookDataProvider } from './contexts/WorkbookDataContext';
import { useWorkbookData } from './hooks/useWorkbookData';
import { NotificationProvider } from './contexts/NotificationContext';
import { useNotification } from './hooks/useNotification';

// Hooks
import { useTabStore } from './hooks/useTabStore';

// Types
import type { Tab } from './types/tabs';
import type { WorkbookData } from './types/import';

// Lazy load tab components for code splitting
const HomeTab = lazy(() => import('./pages/HomeTab'));
const SpreadsheetTab = lazy(() => import('./pages/SpreadsheetTab'));
const FittingTab = lazy(() => import('./pages/FittingTab'));
const SolverTab = lazy(() => import('./pages/SolverTab'));
const MonteCarloTab = lazy(() => import('./pages/MonteCarloTab'));

// Static styles to prevent recreation
const MAIN_CONTAINER_STYLES = {
  display: 'flex',
  flexDirection: 'column',
  height: '100vh',
  width: '100vw',
  margin: 0,
  padding: 0,
  overflow: 'hidden',
  backgroundColor: '#0a0a0a',
  position: 'relative',
} as const;

const CONTENT_AREA_STYLES = {
  display: 'flex',
  flexDirection: 'column',
  flexGrow: 1,
  width: '100%',
  overflow: 'hidden',
  margin: 0,
  padding: 0
} as const;

const TAB_CONTENT_STYLES = {
  flexGrow: 1,
  p: 0,
  bgcolor: '#0a0a0a',
  overflow: 'auto',
  width: '100%',
  margin: 0
} as const;

function App() {
  // Store and state
  const { tabs, activeTabId, addTab: storeAddTab, reorderTabs } = useTabStore();
  
  // Contexts
  const { setPendingWorkbookData } = useWorkbookData();
  const { showNotification } = useNotification();  // Drag state
  const [draggedTab, setDraggedTab] = useState<Tab | null>(null);

  // Ref to store handleAddTab function
  const handleAddTabRef = useRef<((id: string, title: string, content: React.ReactNode, workbookData?: WorkbookData) => void) | null>(null);

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
      case 'montecarlo':
        return <MonteCarloTab />;
      default:
        return <HomeTab openNewTab={safeOpenNewTab} />;
    }
  }, []);

  // Memoized add tab handler
  const handleAddTab = useCallback((id: string, title: string, content: React.ReactNode, workbookData?: WorkbookData) => {
    const tabType = id.split('-')[0] ?? 'home';
    const newTab: Tab = {
      id,
      title,
      content: content ?? createTabContent(tabType, id),
      type: tabType
    };
    storeAddTab(newTab);
    
    // Store workbook data for later loading if provided
    if (workbookData) {
      setPendingWorkbookData(id, workbookData);
    }
  }, [storeAddTab, createTabContent, setPendingWorkbookData]);

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
        type: 'home'
      };
      storeAddTab(homeTab);
    }
  }, [tabs.length, storeAddTab , createTabContent]);  

  // Listen for file open events from file associations
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setupListener = async () => {
      unlisten = await listen<string>('open-file', (event) => {
        const filePath = event.payload;
        console.log('File open requested:', filePath);
        
        // Handle the async operation without returning a promise
        void (async () => {
          try {
            // Import the .anafispread file
            const result = await invoke<{ success: boolean; message?: string; data?: { workbook: WorkbookData } }>(
              'import_anafis_spread_direct',
              { filePath }
            );

            if (result.success && result.data?.workbook) {
              // Always create a new spreadsheet tab for each opened file
              // This allows users to have multiple files open simultaneously
              const fileName = filePath.split('/').pop()?.replace('.anafispread', '') ?? 'Opened File';
              const tabId = `spreadsheet-opened-${Date.now()}`;
              handleAddTabRef.current?.(tabId, fileName, undefined, result.data.workbook);
            } else if (!result.success) {
              // Show error notification for failed import
              const errorMessage = result.message ?? 'Unknown import error';
              showNotification({
                type: 'error',
                message: `Failed to open file "${filePath.split('/').pop()}": ${errorMessage}`
              });
            } else {
              // Show error for successful import but missing workbook data
              showNotification({
                type: 'error',
                message: `Failed to open file "${filePath.split('/').pop()}": Invalid file format or corrupted data`
              });
            }
          } catch (error) {
            console.error('Failed to open file:', error);
            // Show user-facing error notification
            const fileName = filePath.split('/').pop() ?? 'Unknown file';
            const errorMessage = error instanceof Error ? error.message : 'Unknown error occurred';
            showNotification({
              type: 'error',
              message: `Failed to open file "${fileName}": ${errorMessage}`
            });
          }
        })();
      });
    };

    void setupListener();

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [handleAddTab, showNotification]);
  
  // Drag and drop setup
  const sensors = useSensors(useSensor(PointerSensor));

  // Memoized drag handlers
  const handleDragStart = useCallback((event: DragStartEvent) => {
    const tab = tabs.find((t: Tab) => t.id === event.active.id);
    setDraggedTab(tab ?? null);
  }, [tabs]);

  const handleDragEnd = useCallback((event: DragEndEvent) => {
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
  }, [tabs, reorderTabs]);

  // Memoized content area height calculation
  const contentAreaHeight = useMemo(() =>
    'calc(100vh - 96px)'
    , []);

  return (
    <>
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
          <CustomTitleBar
            title='AnaFis'
          />

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
              <OptimizedTabRenderer
                tabs={tabs}
                activeTabId={activeTabId}
              />
            </Box>
          </Box>
        </Box>
      </DndContext>
    </>
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