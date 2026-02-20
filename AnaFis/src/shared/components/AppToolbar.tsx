// Extracted and optimized app toolbar component

import {
  FolderOpen as FolderOpenIcon,
  Save as SaveIcon,
} from '@mui/icons-material';
import {
  AppBar,
  Box,
  Button,
  IconButton,
  Menu,
  MenuItem,
  Toolbar,
} from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import React, { useCallback, useMemo, useRef, useState } from 'react';
import { AddIcon, CalculateIcon, SettingsIcon, StorageIcon } from '@/icons';
import TabButton from '@/shared/components/TabButton';
import { anafisColors } from '@/tabs/spreadsheet/components/sidebar/themes';

interface AppToolbarProps {
  onAddTab: (id: string, title: string, content: React.ReactNode) => void;
  createTabContent: (tabType: string, tabId: string) => React.ReactNode;
}

// Static styles to prevent recreation
const TOOLBAR_STYLES = {
  background:
    'linear-gradient(135deg, rgba(26, 26, 26, 0.95) 0%, rgba(42, 42, 42, 0.95) 100%)',
  backdropFilter: 'blur(20px)',
  borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
  boxShadow: '0 4px 20px rgba(0, 0, 0, 0.3)',
  width: '100%',
} as const;

const PROJECT_BUTTON_STYLES = {
  color: '#ffffff',
  background:
    'linear-gradient(135deg, rgba(33, 150, 243, 0.1) 0%, rgba(33, 150, 243, 0.05) 100%)',
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
    background:
      'linear-gradient(135deg, rgba(33, 150, 243, 0.15) 0%, rgba(33, 150, 243, 0.08) 100%)',
    borderColor: 'rgba(33, 150, 243, 0.5)',
    transform: 'translateY(-1px)',
    boxShadow: '0 4px 16px rgba(33, 150, 243, 0.25)',
    color: anafisColors.primary,
  },
  '&:active': {
    background:
      'linear-gradient(135deg, rgba(33, 150, 243, 0.08) 0%, rgba(33, 150, 243, 0.03) 100%)',
    transform: 'translateY(0px)',
    boxShadow: '0 2px 8px rgba(33, 150, 243, 0.15)',
  },
  '&:focus': {
    outline: 'none',
    boxShadow: '0 0 0 2px rgba(33, 150, 243, 0.3)',
  },
  '& .MuiTouchRipple-root': {
    display: 'none',
  },
} as const;

const MENU_PAPER_STYLES = {
  background:
    'linear-gradient(135deg, rgba(26, 26, 26, 0.98) 0%, rgba(42, 42, 42, 0.98) 100%)',
  backdropFilter: 'blur(20px)',
  border: '1px solid rgba(255, 255, 255, 0.1)',
  borderRadius: 2,
  boxShadow:
    '0 8px 32px rgba(0, 0, 0, 0.4), 0 0 0 1px rgba(255, 255, 255, 0.05)',
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
      color: anafisColors.primary,
      transform: 'translateX(2px)',
    },
  },
} as const;

// Tab button configurations
const TAB_BUTTONS_CONFIG = [
  {
    label: 'Spreadsheet',
    type: 'spreadsheet',
    hoverColor: '#64b5f6',
    hoverBackgroundColor: 'rgba(33, 150, 243, 0.12)',
    hoverBorderColor: 'rgba(33, 150, 243, 0.2)',
    hoverBoxShadowColor: 'rgba(33, 150, 243, 0.3)',
  },
  {
    label: 'Fitting',
    type: 'fitting',
    hoverColor: '#ffb74d',
    hoverBackgroundColor: 'rgba(255, 152, 0, 0.12)',
    hoverBorderColor: 'rgba(255, 152, 0, 0.2)',
    hoverBoxShadowColor: 'rgba(255, 152, 0, 0.3)',
  },
  {
    label: 'Solver',
    type: 'solver',
    hoverColor: '#81c784',
    hoverBackgroundColor: 'rgba(76, 175, 80, 0.12)',
    hoverBorderColor: 'rgba(76, 175, 80, 0.2)',
    hoverBoxShadowColor: 'rgba(76, 175, 80, 0.3)',
  },
  {
    label: 'Monte Carlo',
    type: 'montecarlo',
    hoverColor: '#f06292',
    hoverBackgroundColor: 'rgba(233, 30, 99, 0.12)',
    hoverBorderColor: 'rgba(233, 30, 99, 0.2)',
    hoverBoxShadowColor: 'rgba(233, 30, 99, 0.3)',
  },
] as const;

// Action button configurations
const ACTION_BUTTONS_CONFIG = [
  {
    icon: StorageIcon,
    onClick: 'openDataLibrary',
    title: 'Data Library',
    color: 'rgba(76, 175, 80, 0.06)',
    hoverColor: 'rgba(76, 175, 80, 0.12)',
    activeColor: '#4caf50',
  },
  {
    icon: CalculateIcon,
    onClick: 'openUncertaintyCalculator',
    title: 'Uncertainty Calculator',
    color: 'rgba(156, 39, 176, 0.06)',
    hoverColor: 'rgba(156, 39, 176, 0.12)',
    activeColor: '#9c27b0',
  },
  {
    icon: SettingsIcon,
    onClick: 'openSettings',
    title: 'Settings',
    color: 'rgba(0, 188, 212, 0.06)',
    hoverColor: 'rgba(0, 188, 212, 0.12)',
    activeColor: '#00bcd4',
  },
] as const;

const AppToolbar = React.memo<AppToolbarProps>(
  ({ onAddTab, createTabContent }) => {
    const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);

    // Incremental counter for unique tab IDs
    const tabIdCounter = useRef<number>(0);

    // Memoized event handlers
    const handleProjectMenuClick = useCallback(
      (event: React.MouseEvent<HTMLButtonElement>) => {
        setAnchorEl(event.currentTarget);
      },
      []
    );

    const handleProjectMenuClose = useCallback(() => {
      setAnchorEl(null);
    }, []);

    // Memoized async handlers
    const asyncHandlers = useMemo(
      () => ({
        openDataLibrary: async () => {
          try {
            await invoke('open_data_library_window');
          } catch (error) {
            console.error(
              '[AppToolbar] Failed to open data library window:',
              error
            );
          }
        },
        openUncertaintyCalculator: async () => {
          try {
            await invoke('open_uncertainty_calculator_window');
          } catch (error) {
            console.error(
              '[AppToolbar] Failed to open uncertainty calculator window:',
              error
            );
          }
        },
        openSettings: async () => {
          try {
            await invoke('open_settings_window');
          } catch (error) {
            console.error(
              '[AppToolbar] Failed to open settings window:',
              error
            );
          }
        },
      }),
      []
    );

    // Memoized tab button handlers
    const tabButtonHandlers = useMemo(
      () =>
        TAB_BUTTONS_CONFIG.reduce(
          (acc, config) => {
            acc[config.type] = () => {
              const tabId = `${config.type}-${++tabIdCounter.current}`;
              onAddTab(
                tabId,
                config.label,
                createTabContent(config.type, tabId)
              );
            };
            return acc;
          },
          {} as Record<string, () => void>
        ),
      [onAddTab, createTabContent]
    );

    return (
      <AppBar position="static" sx={TOOLBAR_STYLES}>
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
                  transform: anchorEl ? 'rotate(180deg)' : 'rotate(0deg)',
                  fontSize: '0.9rem',
                  lineHeight: 1,
                  fontWeight: 'bold',
                }}
              >
                ▾
              </Box>
            }
            sx={PROJECT_BUTTON_STYLES}
          >
            Project
          </Button>

          <Menu
            anchorEl={anchorEl}
            open={Boolean(anchorEl)}
            onClose={handleProjectMenuClose}
            disableScrollLock
            hideBackdrop
            slotProps={{ paper: { sx: MENU_PAPER_STYLES } }}
          >
            <MenuItem
              onClick={handleProjectMenuClose}
              sx={{ display: 'flex', alignItems: 'center', gap: 1.5 }}
            >
              <AddIcon sx={{ fontSize: '1.1rem' }} aria-hidden="true" />
              New Project
            </MenuItem>
            <MenuItem
              onClick={handleProjectMenuClose}
              sx={{ display: 'flex', alignItems: 'center', gap: 1.5 }}
            >
              <FolderOpenIcon sx={{ fontSize: '1.1rem' }} aria-hidden="true" />
              Open Project
            </MenuItem>
            <MenuItem
              onClick={handleProjectMenuClose}
              sx={{ display: 'flex', alignItems: 'center', gap: 1.5 }}
            >
              <SaveIcon sx={{ fontSize: '1.1rem' }} aria-hidden="true" />
              Save Project
            </MenuItem>
          </Menu>

          {/* Spacer */}
          <Box sx={{ flexGrow: 1 }} />

          {/* Tab Buttons */}
          {TAB_BUTTONS_CONFIG.map((config) => {
            const handler = tabButtonHandlers[config.type];
            if (!handler) {
              return null;
            } // Skip if handler doesn't exist

            return (
              <TabButton
                key={config.type}
                label={config.label}
                onClick={handler}
                hoverColor={config.hoverColor}
                hoverBackgroundColor={config.hoverBackgroundColor}
                hoverBorderColor={config.hoverBorderColor}
                hoverBoxShadowColor={config.hoverBoxShadowColor}
              />
            );
          })}

          {/* Action Buttons */}
          {ACTION_BUTTONS_CONFIG.map((config) => {
            const IconComponent = config.icon;
            return (
              <IconButton
                key={config.title}
                color="inherit"
                onClick={() => void asyncHandlers[config.onClick]()}
                title={config.title}
                disableRipple
                disableFocusRipple
                sx={{
                  color: '#ffffff',
                  backgroundColor: config.color,
                  border: 'none',
                  borderRadius: 2,
                  mr: 1,
                  transition: 'all 0.18s ease-in-out',
                  '&:hover': {
                    backgroundColor: config.hoverColor,
                    color: config.activeColor,
                    transform: 'scale(1.05)',
                  },
                  '&:focus': {
                    outline: '2px solid rgba(255, 255, 255, 0.8)',
                    outlineOffset: '2px',
                  },
                  '&.Mui-focusVisible': {
                    outline: '2px solid rgba(255, 255, 255, 0.8)',
                    outlineOffset: '2px',
                  },
                }}
              >
                <IconComponent />
              </IconButton>
            );
          })}
        </Toolbar>
      </AppBar>
    );
  }
);

export default AppToolbar;
