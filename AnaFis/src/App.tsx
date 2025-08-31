import React, { useState, useEffect, useCallback } from 'react';
import { DraggableTabBar } from './components/DraggableTabBar';
import { GlobalDragLayer } from './components/GlobalDragLayer';
import CustomTitleBar from './components/CustomTitleBar';
import HomeTab from './pages/HomeTab';
import SpreadsheetTab from './pages/SpreadsheetTab';
import FittingTab from './pages/FittingTab';
import SolverTab from './pages/SolverTab';
import MonteCarloTab from './pages/MonteCarloTab';
import { useTabStore } from './hooks/useTabStore';

import type { Tab } from './types/tabs'; // Import the Tab interface

// TabInfo interface matching the Rust TabInfo struct
interface TabInfo {
  id: string;
  title: string;
  content_type: string;
  state: any;
  icon?: string;
}

// Material-UI Imports
import AppBar from '@mui/material/AppBar';
import Toolbar from '@mui/material/Toolbar';
import Button from '@mui/material/Button';
import Box from '@mui/material/Box';
import Menu from '@mui/material/Menu';
import MenuItem from '@mui/material/MenuItem';
import IconButton from '@mui/material/IconButton';
import AddIcon from '@mui/icons-material/Add';
import SettingsIcon from '@mui/icons-material/Settings';
import CalculateIcon from '@mui/icons-material/Calculate';

// Tauri imports
import { invoke } from '@tauri-apps/api/core';

// Dnd-kit imports
import {
  DndContext,
  DragEndEvent,
  DragStartEvent,
  PointerSensor,
  useSensor,
  useSensors,
} from '@dnd-kit/core';

// IPC imports
import { bus } from './utils/ipc';

interface AppContentProps {
  tabs: Tab[];
  activeTabId: string | null;
  anchorEl: HTMLElement | null;
  handleProjectMenuClick: (event: React.MouseEvent<HTMLButtonElement>) => void;
  handleProjectMenuClose: () => void;
  renderActiveTabContent: () => React.ReactNode;
  addTab: (id: string, title: string, content: React.ReactNode) => void;
  openUncertaintyCalculator: () => void;
  openSettings: () => void;
  isDetachedWindow: boolean;
  isDetachedTabWindow: boolean;
}

function AppContent({
  tabs,
  activeTabId,
  anchorEl,
  handleProjectMenuClick,
  handleProjectMenuClose,
  renderActiveTabContent,
  addTab,
  openUncertaintyCalculator,
  openSettings,
  isDetachedWindow,
  isDetachedTabWindow
}: AppContentProps) {
  // Check if this is a detached window
  // Note: Using props isDetachedWindow and isDetachedTabWindow instead of local state

  useEffect(() => {
    // Check URL parameters to see if this is a detached window
    const urlParams = new URLSearchParams(window.location.search);
    const detached = urlParams.get('detached') === 'true';
    const tabId = urlParams.get('tabId');
    const tabType = urlParams.get('tabType');
    const tabTitle = urlParams.get('tabTitle');

    console.log('URL params:', { detached, tabId, tabType, tabTitle });

    if (detached && tabId && tabType && tabTitle) {
      // This is a detached tab window
      // Auto-create the detached tab
      const decodedTitle = decodeURIComponent(tabTitle);
      if (tabType === 'spreadsheet') {
        addTab(tabId, decodedTitle, <SpreadsheetTab />);
      } else if (tabType === 'fitting') {
        addTab(tabId, decodedTitle, <FittingTab />);
      } else if (tabType === 'solver') {
        addTab(tabId, decodedTitle, <SolverTab />);
      } else if (tabType === 'montecarlo') {
        addTab(tabId, decodedTitle, <MonteCarloTab />);
      }
    } else if (detached) {
      // This is a detached window (but not a tab window)
    }
  }, []); // Remove addTab from dependencies since it should only run once on mount

  const handleReattachTab = async () => {
    try {
      // Get the current tab info from URL parameters
      const urlParams = new URLSearchParams(window.location.search);
      const tabId = urlParams.get('tabId');
      const tabType = urlParams.get('tabType');
      const tabTitle = urlParams.get('tabTitle');

      if (tabId && tabType && tabTitle) {
        // Send tab back to main window
        await invoke('send_tab_to_main', {
          tabId,
          tabType,
          tabTitle: decodeURIComponent(tabTitle)
        });

        // Close this detached window
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const currentWindow = getCurrentWindow();
        await currentWindow.close();
      }
    } catch (error) {
      console.error('Failed to reattach tab:', error);
    }
  };

  // Create a reference to the reattach function for conditional use
  const reattachFunction = isDetachedTabWindow ? handleReattachTab : undefined;

  return (
    <Box sx={{
      display: 'flex',
      flexDirection: 'column',
      height: '100vh',
      width: '100vw',
      margin: 0,
      padding: 0,
      overflow: 'hidden',
      backgroundColor: '#0a0a0a', // Force dark background
    }}
    onContextMenu={(e: React.MouseEvent) => {
      e.preventDefault();
      return false;
    }}
    >
      {/* Custom Title Bar - Show for main window and detached tab windows */}
      {(!isDetachedWindow || isDetachedTabWindow) && (
        <CustomTitleBar
          title={isDetachedTabWindow ? tabs[0]?.title : 'AnaFis'}
          isDetachedTabWindow={isDetachedTabWindow}
          onReattach={reattachFunction}
        />
      )}

      {/* Top Menu Bar / Toolbar - Show for main window and detached tab windows */}
      {!isDetachedWindow && (
        <AppBar
          position="static"
          sx={{
            background: 'linear-gradient(135deg, rgba(26, 26, 26, 0.95) 0%, rgba(42, 42, 42, 0.95) 100%)',
            backdropFilter: 'blur(20px)',
            borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
            boxShadow: '0 4px 20px rgba(0, 0, 0, 0.3)',
            width: '100%',
          }}
        >
          <Toolbar sx={{ gap: 1 }}>
            {/* Project Menu */}
            <Button
              color="inherit"
              onClick={handleProjectMenuClick}
              disableRipple
              disableFocusRipple
              endIcon={
                <Box
                  component="span"
                  sx={{
                    display: 'inline-flex',
                    alignItems: 'center',
                    ml: 0.5,
                    transition: 'transform 0.2s ease-in-out',
                    transform: Boolean(anchorEl) ? 'rotate(180deg)' : 'rotate(0deg)',
                    fontSize: '0.9rem',
                    lineHeight: 1,
                    fontWeight: 'bold',
                  }}
                >
                  ▾
                </Box>
              }
              sx={{
                color: '#ffffff',
                background: 'linear-gradient(135deg, rgba(33, 150, 243, 0.1) 0%, rgba(33, 150, 243, 0.05) 100%)',
                border: '1px solid rgba(33, 150, 243, 0.3)',
                borderRadius: 2,
                px: 2.5,
                py: 0.8,
                fontWeight: 600,
                fontSize: '0.9rem',
                mr: 1,
                minWidth: '120px',
                justifyContent: 'space-between',
                transition: 'all 0.25s cubic-bezier(.2,.8,.2,1)',
                boxShadow: '0 2px 8px rgba(33, 150, 243, 0.15)',
                '&:hover': {
                  background: 'linear-gradient(135deg, rgba(33, 150, 243, 0.15) 0%, rgba(33, 150, 243, 0.08) 100%)',
                  borderColor: 'rgba(33, 150, 243, 0.5)',
                  transform: 'translateY(-1px)',
                  boxShadow: '0 4px 16px rgba(33, 150, 243, 0.25)',
                  color: '#2196f3',
                },
                '&:active': {
                  background: 'linear-gradient(135deg, rgba(33, 150, 243, 0.08) 0%, rgba(33, 150, 243, 0.03) 100%)',
                  transform: 'translateY(0px)',
                  boxShadow: '0 2px 8px rgba(33, 150, 243, 0.15)',
                },
                '&:focus': {
                  outline: 'none',
                  boxShadow: '0 0 0 2px rgba(33, 150, 243, 0.3)',
                },
                // remove ripple node if present
                '& .MuiTouchRipple-root': {
                  display: 'none'
                }
              }}
            >
              Project
            </Button>
            <Menu
              anchorEl={anchorEl}
              open={Boolean(anchorEl)}
              onClose={handleProjectMenuClose}
              disableScrollLock
              hideBackdrop
              slotProps={{
                paper: {
                  sx: {
                    background: 'linear-gradient(135deg, rgba(26, 26, 26, 0.98) 0%, rgba(42, 42, 42, 0.98) 100%)',
                    backdropFilter: 'blur(20px)',
                    border: '1px solid rgba(255, 255, 255, 0.1)',
                    borderRadius: 2,
                    boxShadow: '0 8px 32px rgba(0, 0, 0, 0.4), 0 0 0 1px rgba(255, 255, 255, 0.05)',
                    mt: 0.5,
                    minWidth: '180px',
                    '& .MuiMenuItem-root': {
                      fontSize: '0.9rem',
                      py: 1.5,
                      px: 2,
                      borderRadius: 1,
                      mx: 0.5,
                      my: 0.25,
                      transition: 'all 0.2s ease-in-out',
                      '&:hover': {
                        backgroundColor: 'rgba(33, 150, 243, 0.1)',
                        color: '#2196f3',
                        transform: 'translateX(2px)',
                      },
                    },
                  },
                },
              }}
            >
              <MenuItem
                onClick={handleProjectMenuClose}
                sx={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 1.5,
                  '&:hover': {
                    backgroundColor: 'action.hover',
                    color: 'primary.main',
                  },
                }}
              >
                <AddIcon sx={{ fontSize: '1.1rem' }} />
                New Project
              </MenuItem>
              <MenuItem
                onClick={handleProjectMenuClose}
                sx={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 1.5,
                  '&:hover': {
                    backgroundColor: 'action.hover',
                    color: 'primary.main',
                  },
                }}
              >
                📁 Open Project
              </MenuItem>
              <MenuItem
                onClick={handleProjectMenuClose}
                sx={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 1.5,
                  '&:hover': {
                    backgroundColor: 'action.hover',
                    color: 'primary.main',
                  },
                }}
              >
                💾 Save Project
              </MenuItem>
            </Menu>

            {/* Spacer */}
            <Box sx={{ flexGrow: 1 }} />

            <Button
              color="inherit"
              startIcon={<AddIcon />}
              onClick={() => addTab('spreadsheet-' + Date.now(), '📊 Spreadsheet', <SpreadsheetTab />)}
              sx={{
                color: 'text.secondary',
                borderRadius: 2,
                px: 2,
                transition: 'all 0.2s ease-in-out',
                '&:hover': {
                  backgroundColor: 'rgba(33, 150, 243, 0.12)',
                  color: '#42a5f5',
                  transform: 'translateY(-1px)',
                  boxShadow: '0 4px 12px rgba(33, 150, 243, 0.3)',
                  border: '1px solid rgba(33, 150, 243, 0.2)',
                },
                '&:active': {
                  transform: 'translateY(0px)',
                },
              }}
            >
              Spreadsheet
            </Button>
            <Button
              color="inherit"
              startIcon={<AddIcon />}
              onClick={() => addTab('fitting-' + Date.now(), '📈 Fitting', <FittingTab />)}
              sx={{
                color: 'text.secondary',
                borderRadius: 2,
                px: 2,
                transition: 'all 0.2s ease-in-out',
                '&:hover': {
                  backgroundColor: 'rgba(255, 152, 0, 0.12)',
                  color: '#ffb74d',
                  transform: 'translateY(-1px)',
                  boxShadow: '0 4px 12px rgba(255, 152, 0, 0.3)',
                  border: '1px solid rgba(255, 152, 0, 0.2)',
                },
                '&:active': {
                  transform: 'translateY(0px)',
                },
              }}
            >
              Fitting
            </Button>
                        <Button
              color="inherit"
              startIcon={<AddIcon />}
              onClick={() => addTab('solver-' + Date.now(), '🧮 Solver', <SolverTab />)}
              sx={{
                color: 'text.secondary',
                borderRadius: 2,
                px: 2,
                transition: 'all 0.2s ease-in-out',
                '&:hover': {
                  backgroundColor: 'rgba(76, 175, 80, 0.12)',
                  color: '#81c784',
                  transform: 'translateY(-1px)',
                  boxShadow: '0 4px 12px rgba(76, 175, 80, 0.3)',
                  border: '1px solid rgba(76, 175, 80, 0.2)',
                },
                '&:active': {
                  transform: 'translateY(0px)',
                },
              }}
            >
              Solver
            </Button>
                        <Button
              color="inherit"
              startIcon={<AddIcon />}
              onClick={() => addTab('montecarlo-' + Date.now(), '🎲 Monte Carlo', <MonteCarloTab />)}
              sx={{
                color: 'text.secondary',
                borderRadius: 2,
                px: 2,
                transition: 'all 0.2s ease-in-out',
                '&:hover': {
                  backgroundColor: 'rgba(233, 30, 99, 0.12)',
                  color: '#f06292',
                  transform: 'translateY(-1px)',
                  boxShadow: '0 4px 12px rgba(233, 30, 99, 0.3)',
                  border: '1px solid rgba(233, 30, 99, 0.2)',
                },
                '&:active': {
                  transform: 'translateY(0px)',
                },
              }}
            >
              Monte Carlo
            </Button>

            {/* Uncertainty Calculator Action */}
            <IconButton
              color="inherit"
              onClick={openUncertaintyCalculator}
              title="Uncertainty Calculator"
              disableRipple
              disableFocusRipple
              sx={{
                color: '#ffffff',
                backgroundColor: 'rgba(156, 39, 176, 0.06)',
                border: 'none',
                borderRadius: 2,
                mr: 1,
                transition: 'all 0.18s ease-in-out',
                '&:hover': {
                  backgroundColor: 'rgba(156, 39, 176, 0.12)',
                  color: '#9c27b0',
                  transform: 'scale(1.05)',
                },
                '&:focus': {
                  outline: '2px solid rgba(255, 255, 255, 0.8)',
                  outlineOffset: '2px',
                },
                '&.Mui-focusVisible': {
                  outline: '2px solid rgba(255, 255, 255, 0.8)',
                  outlineOffset: '2px',
                }
              }}
            >
              <CalculateIcon />
            </IconButton>

            {/* Settings Action */}
            <IconButton
              color="inherit"
              title="Settings"
              disableRipple
              disableFocusRipple
              onClick={openSettings}
              sx={{
                color: '#ffffff',
                backgroundColor: 'rgba(0, 188, 212, 0.06)',
                border: 'none',
                borderRadius: 2,
                transition: 'all 0.18s ease-in-out',
                '&:hover': {
                  backgroundColor: 'rgba(0, 188, 212, 0.12)',
                  color: '#00bcd4',
                  transform: 'scale(1.05)',
                },
                '&:focus': {
                  outline: '2px solid rgba(255, 255, 255, 0.8)',
                  outlineOffset: '2px',
                },
                '&.Mui-focusVisible': {
                  outline: '2px solid rgba(255, 255, 255, 0.8)',
                  outlineOffset: '2px',
                }
              }}
            >
              <SettingsIcon />
            </IconButton>
          </Toolbar>
        </AppBar>
      )}      {/* Main Content Area - TabBar and Tab Content */}
      <Box sx={{
        display: 'flex',
        flexDirection: 'column',
        flexGrow: 1,
        width: '100%',
        height: isDetachedWindow ? '100vh' : (isDetachedTabWindow ? 'calc(100vh - 32px)' : 'calc(100vh - 96px)'), // Adjust height for detached windows
        overflow: 'hidden', // Prevent overflow
        margin: 0,
        padding: 0
      }}>
        {/* Show TabBar in main window and detached tab windows */}
        {true && (
          <DraggableTabBar />
        )}
        <Box sx={{
          flexGrow: 1,
          p: 0, // Remove padding
          bgcolor: '#0a0a0a', // Force dark background
          overflow: 'auto', // Allow scrolling for tab content
          width: '100%', // Ensure full width
          margin: 0
        }}> {/* Padding for tab content */}
          {renderActiveTabContent()}
        </Box>
      </Box>
    </Box>
  );
}

function App() {
  const { tabs, activeTabId, addTab: storeAddTab, detachTab } = useTabStore();

  // Drag and drop sensors
  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8,
      },
    })
  );

  // Drag handlers for cross-window functionality
  const handleDragStart = (event: DragStartEvent) => {
    const { active } = event;
    const draggedTab = tabs.find((tab: Tab) => tab.id === active.id);
    if (draggedTab && draggedTab.id !== 'home') {
      // Broadcast drag start to all windows
      bus.emit('tab-drag-start', draggedTab);
    }
  };

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;

    if (!over) {
      // Released outside any tab bar → detach to new window
      const tab = tabs.find((t: Tab) => t.id === active.id);
      if (tab && tab.id !== 'home') {
        // Get the final mouse position from the drag event
        const clientOffset = event.activatorEvent as any;
        const position = clientOffset ? {
          x: Math.round(clientOffset.clientX || 100),
          y: Math.round(clientOffset.clientY || 100)
        } : { x: 100, y: 100 };

        detachTab(tab.id, position); // Create new window at drop position
      }
    } else if (over && active.id !== over.id) {
      // Check if this is a cross-window drop
      const tab = tabs.find((t: Tab) => t.id === active.id);
      if (tab && tab.id !== 'home') {
        // For cross-window drops, emit the tab-drop event
        // This will be picked up by other windows
        bus.emit('tab-drop', tab);
        // Remove from this window
        // Note: removeTab is handled by the store
      } else {
        // Handle reordering within same window
        // Note: reorderTabs is handled by the store
      }
    }
  };

  const handleDragOver = () => {
    // Handle drag over for visual feedback
    // Note: This can be enhanced later for better UX
  };
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null); // For Project Menu
  const [isLoading, setIsLoading] = useState(true);

  // Wrapper function to maintain compatibility with existing interface
  const addTab = useCallback((id: string, title: string, content: React.ReactNode) => {
    const newTab: Tab = { id, title, content };
    storeAddTab(newTab);
  }, [storeAddTab]);

  const openUncertaintyCalculator = async () => {
    try {
      await invoke('open_uncertainty_calculator_window');
    } catch (error) {
      console.error('Failed to open uncertainty calculator window:', error);
    }
  };

  const openSettings = async () => {
    try {
      await invoke('open_settings_window');
    } catch (error) {
      console.error('Failed to open settings window:', error);
    }
  };

  const handleProjectMenuClick = (event: React.MouseEvent<HTMLButtonElement>) => {
    setAnchorEl(event.currentTarget);
  };

  const handleProjectMenuClose = () => {
    setAnchorEl(null);
  };

  // Helper to render active tab content
  const renderActiveTabContent = () => {
    const activeTab = tabs.find(tab => tab.id === activeTabId);
    return activeTab ? activeTab.content : null;
  };

  // Automatically open Home Tab on initial load
  useEffect(() => {
    // Only initialize if we have no tabs and haven't initialized yet
    if (tabs.length === 0) {
      // Check if this is a detached window
      const urlParams = new URLSearchParams(window.location.search);
      const isDetached = urlParams.get('detached') === 'true';
      const detachedTabId = urlParams.get('tabId');
      const detachedTabType = urlParams.get('tabType');
      const detachedTabTitle = urlParams.get('tabTitle');

      if (isDetached && detachedTabType) {
        console.log('Loading detached window for:', { detachedTabId, detachedTabType, detachedTabTitle });

        // Create the appropriate tab based on type
        let tabContent: React.ReactNode;
        let tabTitle: string;

        switch (detachedTabType) {
          case 'spreadsheet':
            tabContent = <SpreadsheetTab />;
            tabTitle = detachedTabTitle ? decodeURIComponent(detachedTabTitle) : '📊 Spreadsheet';
            break;
          case 'fitting':
            tabContent = <FittingTab />;
            tabTitle = detachedTabTitle ? decodeURIComponent(detachedTabTitle) : '📈 Fitting';
            break;
          case 'solver':
            tabContent = <SolverTab />;
            tabTitle = detachedTabTitle ? decodeURIComponent(detachedTabTitle) : '🧮 Solver';
            break;
          case 'montecarlo':
            tabContent = <MonteCarloTab />;
            tabTitle = detachedTabTitle ? decodeURIComponent(detachedTabTitle) : '🎲 Monte Carlo';
            break;
          default:
            tabContent = <HomeTab openNewTab={addTab} />;
            tabTitle = '🏠 Home';
        }

        // Create the tab with the original ID if provided, otherwise generate new one
        const finalTabId = detachedTabId || `${detachedTabType}-${Date.now()}`;
        addTab(finalTabId, tabTitle, tabContent);
      } else {
        // Normal startup - add home tab
        addTab('home', '🏠 Home', <HomeTab openNewTab={addTab} />);
      }
    }
    // Set loading to false after initial render
    setIsLoading(false);
  }, []); // Empty dependency array to run only once

  // Listen for reattach tab events from detached windows
  useEffect(() => {
    const handleReattachTabEvent = (event: any) => {
      const tabInfo = event.payload as TabInfo;
      console.log('Received reattach tab event:', tabInfo);

      // Create the appropriate tab content
      let tabContent: React.ReactNode;
      switch (tabInfo.content_type) {
        case 'spreadsheet':
          tabContent = <SpreadsheetTab />;
          break;
        case 'fitting':
          tabContent = <FittingTab />;
          break;
        case 'solver':
          tabContent = <SolverTab />;
          break;
        case 'montecarlo':
          tabContent = <MonteCarloTab />;
          break;
        default:
          tabContent = <HomeTab openNewTab={addTab} />;
      }

      // Add the tab back to the main window
      addTab(tabInfo.id, tabInfo.title, tabContent);
    };

    // Listen for the reattach event
    const setupListener = async () => {
      try {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const currentWindow = getCurrentWindow();
        currentWindow.listen('reattach-tab', handleReattachTabEvent);
      } catch (error) {
        console.error('Failed to setup reattach listener:', error);
      }
    };

    setupListener();

    // Cleanup
    return () => {
      // Note: Tauri event listeners are automatically cleaned up when component unmounts
    };
  }, [addTab]);

  // Handle click outside to close menu
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (anchorEl && !anchorEl.contains(event.target as Node)) {
        handleProjectMenuClose();
      }
    };

    if (anchorEl) {
      document.addEventListener('mousedown', handleClickOutside);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [anchorEl]);

  return (
    <>
      <DndContext
        sensors={sensors}
        onDragStart={handleDragStart}
        onDragEnd={handleDragEnd}
        onDragOver={handleDragOver}
      >
        <GlobalDragLayer />
        <AppContent
          tabs={tabs}
          activeTabId={activeTabId}
          anchorEl={anchorEl}
          handleProjectMenuClick={handleProjectMenuClick}
          handleProjectMenuClose={handleProjectMenuClose}
          renderActiveTabContent={renderActiveTabContent}
          addTab={addTab}
          openUncertaintyCalculator={openUncertaintyCalculator}
          openSettings={openSettings}
          isDetachedWindow={false}
          isDetachedTabWindow={false}
        />
      </DndContext>

      {/* Loading overlay to prevent flashes */}
      {isLoading && (
        <Box
          sx={{
            position: 'fixed',
            top: 0,
            left: 0,
            width: '100vw',
            height: '100vh',
            backgroundColor: '#0a0a0a',
            zIndex: 9999,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }}
        >
          {/* Loading content can be added here if needed */}
        </Box>
      )}
    </>
  );
}

export default App;