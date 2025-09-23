import React, { useState, useEffect, useCallback } from 'react';
import { DraggableTabBar } from './DraggableTabBar';
import CustomTitleBar from './CustomTitleBar';
import TabButton from './TabButton';
import { getTabContent } from '../utils/tabs';
import SpreadsheetTab from '../pages/SpreadsheetTab';
import FittingTab from '../pages/FittingTab';
import SolverTab from '../pages/SolverTab';
import MonteCarloTab from '../pages/MonteCarloTab';
import { useTabStore } from '../hooks/useTabStore';

import type { Tab, TabFromDetachedPayload } from '../types/tabs';

// Material-UI Imports
import AppBar from '@mui/material/AppBar';
import Toolbar from '@mui/material/Toolbar';
import Button from '@mui/material/Button';
import Box from '@mui/material/Box';
import Menu from '@mui/material/Menu';
import MenuItem from '@mui/material/MenuItem';
import IconButton from '@mui/material/IconButton';
import { AddIcon, SettingsIcon, CalculateIcon } from '../icons';

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

import { getTabIcon } from '../utils/tabColors';

// Helper function to get drag overlay icon
const getDragOverlayIcon = (tabId: string) => {
  return getTabIcon(tabId, '1rem');
};

export const MainWindow = () => {
  const { tabs, activeTabId, addTab: storeAddTab, detachTab, reorderTabs } = useTabStore();
  const [draggedTab, setDraggedTab] = useState<Tab | null>(null);

  const sensors = useSensors(
    useSensor(PointerSensor, {})
  );

  const handleDragStart = (event: DragStartEvent) => {
    const tab = tabs.find((t: Tab) => t.id === event.active.id);
    setDraggedTab(tab || null);
  };

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over, delta } = event;
    setDraggedTab(null);
    const tab = tabs.find((t: Tab) => t.id === active.id);
    if (!tab) return;

    const verticalThreshold = 80;
    const shouldDetachVertical = Math.abs(delta.y) > verticalThreshold;
    const shouldDetachNoTarget = !over;

    if ((shouldDetachVertical || shouldDetachNoTarget) && tab.id !== 'home') {
      let position = { x: 100, y: 100 };
      if (event.activatorEvent) {
        const initialEvent = event.activatorEvent as MouseEvent;
        position = {
          x: Math.round(initialEvent.clientX + delta.x),
          y: Math.round(initialEvent.clientY + delta.y)
        };
      }
      detachTab(tab.id, position);
    } else if (over && active.id !== over.id) {
      const activeIndex = tabs.findIndex((tab) => tab.id === active.id);
      const overIndex = tabs.findIndex((tab) => tab.id === over.id);
      if (activeIndex !== -1 && overIndex !== -1) {
        reorderTabs(activeIndex, overIndex);
      }
    }
  };

  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);

  const addTab = useCallback((id: string, title: string, content: React.ReactNode) => {
    const newTab: Tab = { id, title, content };
    storeAddTab(newTab);
  }, [storeAddTab]);

  const openUncertaintyCalculator = async () => {
    try {
      await invoke('open_uncertainty_calculator_window');
    } catch (error) {
      console.error("Failed to open uncertainty calculator window", error);
    }
  };

  const openSettings = async () => {
    try {
      await invoke('open_settings_window');
    } catch (error) {
      console.error("Failed to open settings window", error);
    }
  };

  const handleProjectMenuClick = (event: React.MouseEvent<HTMLButtonElement>) => {
    setAnchorEl(event.currentTarget);
  };

  const handleProjectMenuClose = () => {
    setAnchorEl(null);
  };

  const renderActiveTabContent = () => {
    const activeTab = tabs.find(tab => tab.id === activeTabId);
    if (!activeTab) return null;
    return activeTab.content;
  };

  useEffect(() => {
    if (tabs.length === 0) {
      addTab('home', 'Home', getTabContent('home', addTab));
    }
  }, [addTab, tabs.length]);

  useEffect(() => {
    const handleReattachTabEvent = (event: { payload: unknown }) => {
      const tabInfo = event.payload as TabFromDetachedPayload;
      const tabContent = getTabContent(tabInfo.content_type, addTab);
      addTab(tabInfo.id, tabInfo.title, tabContent);
    };

    const setupListener = async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event');
        await listen('tab-from-detached', handleReattachTabEvent);
      } catch (error) {
        console.error("Failed to setup reattach listener", error);
      }
    };
    setupListener();
  }, [addTab]);

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
                backgroundColor: '#9c27b0',
                border: `2px solid #ba68c8`,
                color: '#ffffff',
                boxShadow: '0 8px 25px rgba(0, 0, 0, 0.4), 0 0 0 1px rgba(255, 255, 255, 0.1)',
                cursor: 'grabbing',
                transform: 'rotate(3deg) scale(1.05)',
                zIndex: 9999,
                backdropFilter: 'blur(10px)',
              }}
            >
              <Box sx={{ mr: 1, display: 'flex', alignItems: 'center', flexShrink: 0 }}>
                {getDragOverlayIcon(draggedTab.id)}
              </Box>
              <Box sx={{ flex: 1, fontWeight: 600, fontSize: '0.85rem', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis', mr: 1, letterSpacing: '0.025em' }}>
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
          backgroundColor: '#0a0a0a',
          position: 'relative',
        }}
          onContextMenu={(e: React.MouseEvent) => {
            e.preventDefault();
            return false;
          }}
        >
          <CustomTitleBar title="AnaFis" />
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
              <Button
                color="inherit"
                onClick={handleProjectMenuClick}
                endIcon={<Box component="span" sx={{ display: 'inline-flex', alignItems: 'center', ml: 0.5, transition: 'transform 0.2s ease-in-out', transform: Boolean(anchorEl) ? 'rotate(180deg)' : 'rotate(0deg)', fontSize: '0.9rem', lineHeight: 1, fontWeight: 'bold' }}>▾</Box>}
                sx={{ color: '#ffffff', background: 'linear-gradient(135deg, rgba(33, 150, 243, 0.1) 0%, rgba(33, 150, 243, 0.05) 100%)', border: '1px solid rgba(33, 150, 243, 0.3)', borderRadius: 2, px: 2.5, py: 0.8, fontWeight: 600, fontSize: '0.9rem', mr: 1, minWidth: '120px', justifyContent: 'space-between', transition: 'all 0.25s cubic-bezier(.2,.8,.2,1)', boxShadow: '0 2px 8px rgba(33, 150, 243, 0.15)', '&:hover': { background: 'linear-gradient(135deg, rgba(33, 150, 243, 0.15) 0%, rgba(33, 150, 243, 0.08) 100%)', borderColor: 'rgba(33, 150, 243, 0.5)', transform: 'translateY(-1px)', boxShadow: '0 4px 16px rgba(33, 150, 243, 0.25)', color: '#2196f3', }, '&:active': { background: 'linear-gradient(135deg, rgba(33, 150, 243, 0.08) 0%, rgba(33, 150, 243, 0.03) 100%)', transform: 'translateY(0px)', boxShadow: '0 2px 8px rgba(33, 150, 243, 0.15)', }, '&:focus': { outline: 'none', boxShadow: '0 0 0 2px rgba(33, 150, 243, 0.3)', }, '& .MuiTouchRipple-root': { display: 'none' } }}
              >
                Project
              </Button>
              <Menu
                anchorEl={anchorEl}
                open={Boolean(anchorEl)}
                onClose={handleProjectMenuClose}
                disableScrollLock
                hideBackdrop
                slotProps={{ paper: { sx: { background: 'linear-gradient(135deg, rgba(26, 26, 26, 0.98) 0%, rgba(42, 42, 42, 0.98) 100%)', backdropFilter: 'blur(20px)', border: '1px solid rgba(255, 255, 255, 0.1)', borderRadius: 2, boxShadow: '0 8px 32px rgba(0, 0, 0, 0.4), 0 0 0 1px rgba(255, 255, 255, 0.05)', mt: 0.5, minWidth: '180px', '& .MuiMenuItem-root': { fontSize: '0.9rem', py: 1.5, px: 2, borderRadius: 1, mx: 0.5, my: 0.25, transition: 'all 0.2s ease-in-out', '&:hover': { backgroundColor: 'rgba(33, 150, 243, 0.1)', color: '#2196f3', transform: 'translateX(2px)', }, }, }, }, }}
              >
                <MenuItem onClick={handleProjectMenuClose} sx={{ display: 'flex', alignItems: 'center', gap: 1.5, '&:hover': { backgroundColor: 'action.hover', color: 'primary.main', }, }}>
                  <AddIcon sx={{ fontSize: '1.1rem' }} />
                  New Project
                </MenuItem>
                <MenuItem onClick={handleProjectMenuClose} sx={{ display: 'flex', alignItems: 'center', gap: 1.5, '&:hover': { backgroundColor: 'action.hover', color: 'primary.main', }, }}>
                  📁 Open Project
                </MenuItem>
                <MenuItem onClick={handleProjectMenuClose} sx={{ display: 'flex', alignItems: 'center', gap: 1.5, '&:hover': { backgroundColor: 'action.hover', color: 'primary.main', }, }}>
                  💾 Save Project
                </MenuItem>
              </Menu>
              <Box sx={{ flexGrow: 1 }} />
              <TabButton label="Spreadsheet" onClick={() => addTab('spreadsheet-' + Date.now(), 'Spreadsheet', <SpreadsheetTab />)} hoverColor="#64b5f6" hoverBackgroundColor="rgba(33, 150, 243, 0.12)" hoverBorderColor="rgba(33, 150, 243, 0.2)" hoverBoxShadowColor="rgba(33, 150, 243, 0.3)" />
              <TabButton label="Fitting" onClick={() => addTab('fitting-' + Date.now(), 'Fitting', <FittingTab />)} hoverColor="#ffb74d" hoverBackgroundColor="rgba(255, 152, 0, 0.12)" hoverBorderColor="rgba(255, 152, 0, 0.2)" hoverBoxShadowColor="rgba(255, 152, 0, 0.3)" />
              <TabButton label="Solver" onClick={() => addTab('solver-' + Date.now(), 'Solver', <SolverTab />)} hoverColor="#81c784" hoverBackgroundColor="rgba(76, 175, 80, 0.12)" hoverBorderColor="rgba(76, 175, 80, 0.2)" hoverBoxShadowColor="rgba(76, 175, 80, 0.3)" />
              <TabButton label="Monte Carlo" onClick={() => addTab('montecarlo-' + Date.now(), 'Monte Carlo', <MonteCarloTab />)} hoverColor="#f06292" hoverBackgroundColor="rgba(233, 30, 99, 0.12)" hoverBorderColor="rgba(233, 30, 99, 0.2)" hoverBoxShadowColor="rgba(233, 30, 99, 0.3)" />
              <IconButton color="inherit" onClick={openUncertaintyCalculator} title="Uncertainty Calculator" sx={{ color: '#ffffff', backgroundColor: 'rgba(156, 39, 176, 0.06)', border: 'none', borderRadius: 2, mr: 1, transition: 'all 0.18s ease-in-out', '&:hover': { backgroundColor: 'rgba(156, 39, 176, 0.12)', color: '#9c27b0', transform: 'scale(1.05)', }, '&:focus': { outline: '2px solid rgba(255, 255, 255, 0.8)', outlineOffset: '2px', }, '&.Mui-focusVisible': { outline: '2px solid rgba(255, 255, 255, 0.8)', outlineOffset: '2px', } }}>
                <CalculateIcon />
              </IconButton>
              <IconButton color="inherit" title="Settings" onClick={openSettings} sx={{ color: '#ffffff', backgroundColor: 'rgba(0, 188, 212, 0.06)', border: 'none', borderRadius: 2, transition: 'all 0.18s ease-in-out', '&:hover': { backgroundColor: 'rgba(0, 188, 212, 0.12)', color: '#00bcd4', transform: 'scale(1.05)', }, '&:focus': { outline: '2px solid rgba(255, 255, 255, 0.8)', outlineOffset: '2px', }, '&.Mui-focusVisible': { outline: '2px solid rgba(255, 255, 255, 0.8)', outlineOffset: '2px', } }}>
                <SettingsIcon />
              </IconButton>
            </Toolbar>
          </AppBar>
          <Box sx={{ display: 'flex', flexDirection: 'column', flexGrow: 1, width: '100%', height: 'calc(100vh - 96px)', overflow: 'hidden', margin: 0, padding: 0 }}>
            <DraggableTabBar />
            <Box sx={{ flexGrow: 1, p: 0, bgcolor: '#0a0a0a', overflow: 'auto', width: '100%', margin: 0 }}>
              {renderActiveTabContent()}
            </Box>
          </Box>
        </Box>
      </DndContext>
    </>
  );
};
