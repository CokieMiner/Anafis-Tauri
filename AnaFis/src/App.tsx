import React, { useState, useEffect, useCallback } from 'react';
import { DraggableTabBar } from './components/DraggableTabBar';
import CustomTitleBar from './components/CustomTitleBar';
import TabButton from './components/TabButton';
import HomeTab from './pages/HomeTab';
import SpreadsheetTab from './pages/SpreadsheetTab';
import FittingTab from './pages/FittingTab';
import SolverTab from './pages/SolverTab';
import MonteCarloTab from './pages/MonteCarloTab';
import { useTabStore } from './hooks/useTabStore';
import { DetachedTabWindow } from './components/DetachedTabWindow';

// Material-UI Icons for drag overlay
// Icons now handled by shared utility function

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
import { AddIcon, SettingsIcon, CalculateIcon } from './icons';

// Tauri imports
import { invoke } from '@tauri-apps/api/core';

// Dnd-kit imports
import {
  DndContext,
  DragEndEvent,
  DragStartEvent,
  DragOverlay,
  PointerSensor,
  useSensor,
  useSensors,
  closestCenter,
} from '@dnd-kit/core';

import { getTabIcon } from './utils/tabColors';

// Helper function to get drag overlay icon
const getDragOverlayIcon = (tabId: string) => {
  return getTabIcon(tabId, '1rem');
};


function App() {
  const { tabs, activeTabId, addTab: storeAddTab, detachTab, reorderTabs } = useTabStore();
  const [isDetachedTabWindow, setIsDetachedTabWindow] = useState(false);
  const [draggedTab, setDraggedTab] = useState<Tab | null>(null);

  // Drag and drop sensors with horizontal preference
  const sensors = useSensors(
    useSensor(PointerSensor, {
      // Remove activation constraints for testing
    })
  );

  // Simplified drag handlers for tab reordering and detachment
  const handleDragStart = (event: DragStartEvent) => {
    // Set the dragged tab for the overlay
    const tab = tabs.find((t: Tab) => t.id === event.active.id);
    setDraggedTab(tab || null);
  };



  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over, delta } = event;

    // Clear the dragged tab
    setDraggedTab(null);

    // Get the tab being dragged
    const tab = tabs.find((t: Tab) => t.id === active.id);
    if (!tab) {
      return;
    }

    // Check if we should detach based on vertical movement or being outside tab bar
    const verticalThreshold = 80; // pixels
    const shouldDetachVertical = Math.abs(delta.y) > verticalThreshold;
    const shouldDetachNoTarget = !over;

    if ((shouldDetachVertical || shouldDetachNoTarget) && tab.id !== 'home') {
      // Calculate final mouse position from initial position + drag delta
      let position = { x: 100, y: 100 }; // Default fallback

      if (event.activatorEvent) {
        const initialEvent = event.activatorEvent as MouseEvent;
        position = {
          x: Math.round(initialEvent.clientX + delta.x),
          y: Math.round(initialEvent.clientY + delta.y)
        };
      }

      detachTab(tab.id, position);
    } else if (over && active.id !== over.id) {
      // Handle reordering within same window
      const activeIndex = tabs.findIndex((tab) => tab.id === active.id);
      const overIndex = tabs.findIndex((tab) => tab.id === over.id);

      if (activeIndex !== -1 && overIndex !== -1) {
        reorderTabs(activeIndex, overIndex);
      }
    }
  };

  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null); // For Project Menu

  // Wrapper function to maintain compatibility with existing interface
  const addTab = useCallback((id: string, title: string, content: React.ReactNode) => {
    const newTab: Tab = { id, title, content };
    storeAddTab(newTab);
  }, [storeAddTab]);

  const openUncertaintyCalculator = async () => {
    try {
      await invoke('open_uncertainty_calculator_window');
    } catch (error) {
      // Failed to open uncertainty calculator window
    }
  };

  const openSettings = async () => {
    try {
      await invoke('open_settings_window');
    } catch (error) {
      // Failed to open settings window
    }
  };

  const handleProjectMenuClick = (event: React.MouseEvent<HTMLButtonElement>) => {
    setAnchorEl(event.currentTarget);
  };

  const handleProjectMenuClose = () => {
    setAnchorEl(null);
  };

  const handleReattachTab = async () => {
    try {
      // Get the current tab info from URL parameters
      const urlParams = new URLSearchParams(window.location.search);
      const tabId = urlParams.get('tabId');
      const tabType = urlParams.get('tabType');
      const tabTitle = urlParams.get('tabTitle');

      if (tabId && tabType && tabTitle) {
        // Close this detached window: get instance first
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const currentWindow = getCurrentWindow();

        // Send tab back to main window (include both key styles for compatibility)
        await invoke('send_tab_to_main', {
          tab_info: {
            id: tabId,
            title: decodeURIComponent(tabTitle),
            content_type: tabType,
            state: {},
            icon: null
          },
          tabInfo: {
            id: tabId,
            title: decodeURIComponent(tabTitle),
            content_type: tabType,
            state: {},
            icon: null
          },
          window_id: currentWindow.label,
          windowId: currentWindow.label
        });

        await currentWindow.close();
      }
    } catch (error) {
      // Failed to reattach tab
    }
  };

  // Use handleReattachTab directly when rendering CustomTitleBar for detached windows

  // Helper to render active tab content
  const renderActiveTabContent = () => {
    const activeTab = tabs.find(tab => tab.id === activeTabId);
    if (!activeTab) return null;

    if (isDetachedTabWindow) {
      return (
        <DetachedTabWindow>
          {activeTab.content}
        </DetachedTabWindow>
      );
    }

    return activeTab.content;
  };

  // Check if this is a detached window
  useEffect(() => {
    const checkWindowType = async () => {
      try {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const currentWindow = getCurrentWindow();
        if (currentWindow.label === 'global_drag_preview') {
          // Skip drag preview window logic - not needed
          return;
        }
      } catch (error) {
        // Failed to check window type
      }
    };

    checkWindowType();
  }, []);

  // Automatically open Home Tab on initial load
  useEffect(() => {
    // Only initialize if we have no tabs and haven't initialized yet
    if (tabs.length === 0) {
      // Check if this is a detached window by reading URL params
      (async () => {
        try {
          const urlParams = new URLSearchParams(window.location.search);
          const tabId = urlParams.get('tabId');
          const tabType = urlParams.get('tabType');
          const tabTitle = urlParams.get('tabTitle');

          if (tabId && tabType && tabTitle) {
            setIsDetachedTabWindow(true);

            let tabContent: React.ReactNode;
            switch (tabType) {
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

            addTab(tabId, decodeURIComponent(tabTitle), tabContent);
          } else {
            // Normal startup - add home tab
            addTab('home', 'Home', <HomeTab openNewTab={addTab} />);
          }
        } catch (error) {
          // Fallback - add home tab
          addTab('home', 'Home', <HomeTab openNewTab={addTab} />);
        }
      })();
    }
  }, [addTab]); // Removed isDragPreviewWindow dependency

  // Listen for reattach tab events from detached windows
  useEffect(() => {
    const handleReattachTabEvent = (event: any) => {
      const tabInfo = event.payload as TabInfo;

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
        const { listen } = await import('@tauri-apps/api/event');
        // Listen for event emitted by Rust when a detached tab is sent back
        await listen('tab-from-detached', handleReattachTabEvent);
      } catch (error) {
        // Failed to setup reattach listener
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
        collisionDetection={closestCenter}
        onDragStart={handleDragStart}

        onDragEnd={handleDragEnd}
      >
        <DragOverlay>
          {draggedTab ? (
            <Box
              sx={{
                position: 'relative',
                display: 'flex',
                alignItems: 'center',
                minWidth: '200px',
                maxWidth: '200px',
                height: '44px',
                padding: '8px 12px',
                borderRadius: '8px',
                backgroundColor: draggedTab.id === 'home' ? '#9c27b0' :
                  draggedTab.id.includes('optimized-spreadsheet') ? '#2196f3' :
                    draggedTab.id.includes('spreadsheet') ? '#2196f3' :
                      draggedTab.id.includes('fitting') ? '#ff9800' :
                        draggedTab.id.includes('solver') ? '#4caf50' :
                          draggedTab.id.includes('montecarlo') ? '#e91e63' : '#9c27b0',
                border: `2px solid ${draggedTab.id === 'home' ? '#ba68c8' :
                  draggedTab.id.includes('optimized-spreadsheet') ? '#64b5f6' :
                    draggedTab.id.includes('spreadsheet') ? '#64b5f6' :
                      draggedTab.id.includes('fitting') ? '#ffb74d' :
                        draggedTab.id.includes('solver') ? '#81c784' :
                          draggedTab.id.includes('montecarlo') ? '#f06292' : '#ba68c8'
                  }`,
                color: '#ffffff',
                boxShadow: '0 8px 25px rgba(0, 0, 0, 0.4), 0 0 0 1px rgba(255, 255, 255, 0.1)',
                cursor: 'grabbing',
                transform: 'rotate(3deg) scale(1.05)',
                zIndex: 9999,
                backdropFilter: 'blur(10px)',
              }}
            >
              {/* Drag Handle */}
              {draggedTab.id !== 'home' && (
                <Box
                  sx={{
                    display: 'flex !important',
                    alignItems: 'center',
                    justifyContent: 'center',
                    width: '24px',
                    height: '28px',
                    marginRight: '6px',
                    cursor: 'grabbing',
                    flexShrink: 0,
                    opacity: '1 !important',
                    visibility: 'visible !important',
                  }}
                >
                  <Box
                    sx={{
                      display: 'flex',
                      flexDirection: 'column',
                      gap: '3px',
                      alignItems: 'center',
                      justifyContent: 'center',
                      width: '100%',
                      height: '100%',
                    }}
                  >
                    <Box sx={{
                      width: '18px',
                      height: '3px',
                      backgroundColor: `${draggedTab.id === 'home' ? '#ba68c8' :
                        draggedTab.id.includes('spreadsheet') ? '#64b5f6' :
                          draggedTab.id.includes('fitting') ? '#ffb74d' :
                            draggedTab.id.includes('solver') ? '#81c784' :
                              draggedTab.id.includes('montecarlo') ? '#f06292' : '#ba68c8'
                        } !important`,
                      borderRadius: '2px',
                      boxShadow: '0 1px 2px rgba(0, 0, 0, 0.5)',
                    }} />
                    <Box sx={{
                      width: '18px',
                      height: '3px',
                      backgroundColor: `${draggedTab.id === 'home' ? '#ba68c8' :
                        draggedTab.id.includes('spreadsheet') ? '#64b5f6' :
                          draggedTab.id.includes('fitting') ? '#ffb74d' :
                            draggedTab.id.includes('solver') ? '#81c784' :
                              draggedTab.id.includes('montecarlo') ? '#f06292' : '#ba68c8'
                        } !important`,
                      borderRadius: '2px',
                      boxShadow: '0 1px 2px rgba(0, 0, 0, 0.5)',
                    }} />
                    <Box sx={{
                      width: '18px',
                      height: '3px',
                      backgroundColor: `${draggedTab.id === 'home' ? '#ba68c8' :
                        draggedTab.id.includes('spreadsheet') ? '#64b5f6' :
                          draggedTab.id.includes('fitting') ? '#ffb74d' :
                            draggedTab.id.includes('solver') ? '#81c784' :
                              draggedTab.id.includes('montecarlo') ? '#f06292' : '#ba68c8'
                        } !important`,
                      borderRadius: '2px',
                      boxShadow: '0 1px 2px rgba(0, 0, 0, 0.5)',
                    }} />
                  </Box>
                </Box>
              )}

              {/* Icon */}
              <Box sx={{ mr: 1, display: 'flex', alignItems: 'center', flexShrink: 0 }}>
                {getDragOverlayIcon(draggedTab.id)}
              </Box>

              {/* Title */}
              <Box sx={{
                flex: 1,
                fontWeight: 600,
                fontSize: '0.85rem',
                whiteSpace: 'nowrap',
                overflow: 'hidden',
                textOverflow: 'ellipsis',
                mr: 1,
                letterSpacing: '0.025em',
              }}>
                {draggedTab.title}
              </Box>
            </Box>
          ) : null}
        </DragOverlay>
        <Box sx={{
          display: 'flex',
          flexDirection: 'column',
          height: '100vh',
          width: '100vw',
          margin: 0,
          padding: 0,
          overflow: 'hidden',
          backgroundColor: '#0a0a0a', // Force dark background
          position: 'relative', // For absolute positioning of drop zone
        }}
          onContextMenu={(e: React.MouseEvent) => {
            e.preventDefault();
            return false;
          }}
        >
          {/* Custom Title Bar - Show for main window and detached tab windows */}
          {(!false || isDetachedTabWindow) && (
            <CustomTitleBar
              title={isDetachedTabWindow ? tabs[0]?.title : 'AnaFis'}
              isDetachedTabWindow={isDetachedTabWindow}
              onReattach={isDetachedTabWindow ? handleReattachTab : undefined}
            />
          )}

          {/* Top Menu Bar / Toolbar - Show only for main window */}
          {!false && !isDetachedTabWindow && (
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

                <TabButton
                  label="Spreadsheet"
                  onClick={() => addTab('spreadsheet-' + Date.now(), 'Spreadsheet', <SpreadsheetTab />)}
                  hoverColor="#64b5f6"
                  hoverBackgroundColor="rgba(33, 150, 243, 0.12)"
                  hoverBorderColor="rgba(33, 150, 243, 0.2)"
                  hoverBoxShadowColor="rgba(33, 150, 243, 0.3)"
                />
                <TabButton
                  label="Fitting"
                  onClick={() => addTab('fitting-' + Date.now(), 'Fitting', <FittingTab />)}
                  hoverColor="#ffb74d"
                  hoverBackgroundColor="rgba(255, 152, 0, 0.12)"
                  hoverBorderColor="rgba(255, 152, 0, 0.2)"
                  hoverBoxShadowColor="rgba(255, 152, 0, 0.3)"
                />
                <TabButton
                  label="Solver"
                  onClick={() => addTab('solver-' + Date.now(), 'Solver', <SolverTab />)}
                  hoverColor="#81c784"
                  hoverBackgroundColor="rgba(76, 175, 80, 0.12)"
                  hoverBorderColor="rgba(76, 175, 80, 0.2)"
                  hoverBoxShadowColor="rgba(76, 175, 80, 0.3)"
                />
                <TabButton
                  label="Monte Carlo"
                  onClick={() => addTab('montecarlo-' + Date.now(), 'Monte Carlo', <MonteCarloTab />)}
                  hoverColor="#f06292"
                  hoverBackgroundColor="rgba(233, 30, 99, 0.12)"
                  hoverBorderColor="rgba(233, 30, 99, 0.2)"
                  hoverBoxShadowColor="rgba(233, 30, 99, 0.3)"
                />

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
          )}          {/* Main Content Area - TabBar and Tab Content */}
          <Box sx={{
            display: 'flex',
            flexDirection: 'column',
            flexGrow: 1,
            width: '100%',
            height: false ? '100vh' : (isDetachedTabWindow ? '100%' : 'calc(100vh - 96px)'), // Adjust height for detached windows
            overflow: 'hidden', // Prevent overflow
            margin: 0,
            padding: 0
          }}>
            {/* Show TabBar only in main window, not in detached tab windows */}
            {!isDetachedTabWindow && (
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
      </DndContext>
    </>
  );
}

export default App;