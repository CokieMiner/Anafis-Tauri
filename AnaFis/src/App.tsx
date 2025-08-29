import React, { useState, useEffect } from 'react';
import TabBar from './components/TabBar';
import CustomTitleBar from './components/CustomTitleBar';
import HomeTab from './pages/HomeTab';
import SpreadsheetTab from './pages/SpreadsheetTab';
import FittingTab from './pages/FittingTab';
import SolverTab from './pages/SolverTab';
import MonteCarloTab from './pages/MonteCarloTab';

import type { Tab } from './types/tabs'; // Import the Tab interface

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

interface AppContentProps {
  tabs: Tab[];
  activeTabId: string | null;
  setActiveTabId: (id: string | null) => void;
  removeTab: (id: string) => void;
  reorderTabs: (sourceIndex: number, destinationIndex: number) => void;
  anchorEl: HTMLElement | null;
  handleProjectMenuClick: (event: React.MouseEvent<HTMLButtonElement>) => void;
  handleProjectMenuClose: () => void;
  renderActiveTabContent: () => React.ReactNode;
  addTab: (id: string, title: string, content: React.ReactNode) => void;
  openUncertaintyCalculator: () => void;
  openSettings: () => void;
}

function AppContent({
  tabs,
  activeTabId,
  setActiveTabId,
  removeTab,
  reorderTabs,
  anchorEl,
  handleProjectMenuClick,
  handleProjectMenuClose,
  renderActiveTabContent,
  addTab,
  openUncertaintyCalculator,
  openSettings
}: AppContentProps) {
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
    }}>
      {/* Custom Title Bar */}
      <CustomTitleBar />

      {/* Top Menu Bar / Toolbar */}
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

      {/* Main Content Area - TabBar and Tab Content */}
      <Box sx={{
        display: 'flex',
        flexDirection: 'column',
        flexGrow: 1,
        width: '100%',
        height: 'calc(100vh - 96px)', // Subtract AppBar (64px) + CustomTitleBar (32px) height
        overflow: 'hidden', // Prevent overflow
        margin: 0,
        padding: 0
      }}>
        <TabBar
          tabs={tabs}
          activeTabId={activeTabId}
          setActiveTabId={setActiveTabId}
          removeTab={removeTab}
          onReorderTabs={reorderTabs}
        />
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
  const [tabs, setTabs] = useState<Tab[]>([]);
  const [activeTabId, setActiveTabId] = useState<string | null>(null);
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null); // For Project Menu
  const [isLoading, setIsLoading] = useState(true);

  const addTab = (id: string, title: string, content: React.ReactNode) => {
    setTabs((prevTabs) => {
      if (prevTabs.some(tab => tab.id === id)) {
        setActiveTabId(id);
        return prevTabs;
      }
      const newTab: Tab = { id, title, content };
      return [...prevTabs, newTab];
    });
    setActiveTabId(id);
  };

  const removeTab = (id: string) => {
    if (id === 'home') {
      return;
    }
    setTabs((prevTabs) => {
      const updatedTabs = prevTabs.filter((tab) => tab.id !== id);
      if (activeTabId === id) {
        const remainingTabs = prevTabs.filter((tab) => tab.id !== id);
        setActiveTabId(remainingTabs[0]?.id || null);
      }
      return updatedTabs;
    });
  };

  const reorderTabs = (sourceIndex: number, destinationIndex: number) => {
    setTabs((prevTabs) => {
      const newTabs = [...prevTabs];
      const [removed] = newTabs.splice(sourceIndex, 1);
      newTabs.splice(destinationIndex, 0, removed);
      return newTabs;
    });
  };

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
    if (tabs.length === 0) { // Ensure it only runs once on initial empty state
      addTab('home', '🏠 Home', <HomeTab openNewTab={addTab} />);
    }
    // Set loading to false after initial render
    setIsLoading(false);
  }, [tabs]); // Depend on tabs to prevent re-adding if tabs are somehow cleared

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
      <AppContent
        tabs={tabs}
        activeTabId={activeTabId}
        setActiveTabId={setActiveTabId}
        removeTab={removeTab}
        reorderTabs={reorderTabs}
        anchorEl={anchorEl}
        handleProjectMenuClick={handleProjectMenuClick}
        handleProjectMenuClose={handleProjectMenuClose}
        renderActiveTabContent={renderActiveTabContent}
        addTab={addTab}
        openUncertaintyCalculator={openUncertaintyCalculator}
        openSettings={openSettings}
      />

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