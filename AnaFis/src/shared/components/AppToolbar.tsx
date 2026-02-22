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
import { anafisTheme } from '@/shared/theme/unifiedTheme';

// Re-export anafisColors for backwards compatibility with TabButton
const anafisColors = {
  primary: anafisTheme.colors.primary.main,
};

interface AppToolbarProps {
  onAddTab: (id: string, title: string, content: React.ReactNode) => void;
  createTabContent: (tabType: string, tabId: string) => React.ReactNode;
}

// Static styles to prevent recreation - using unified theme
const TOOLBAR_STYLES = {
  background: anafisTheme.gradients.toolbar,
  backdropFilter: 'blur(20px)',
  borderBottom: `1px solid ${anafisTheme.colors.border.light}`,
  boxShadow: anafisTheme.shadows.lg,
  width: '100%',
} as const;

const PROJECT_BUTTON_STYLES = {
  color: anafisTheme.colors.text.primary,
  background: `linear-gradient(135deg, ${anafisTheme.colors.sidebar.primary}1A 0%, ${anafisTheme.colors.sidebar.primary}0D 100%)`,
  border: `1px solid ${anafisTheme.colors.sidebar.primary}4D`,
  borderRadius: 2,
  px: 2.5,
  py: 0.8,
  fontWeight: 600,
  fontSize: '0.9rem',
  mr: 1,
  minWidth: '120px',
  justifyContent: 'space-between',
  transition: anafisTheme.transitions.default,
  boxShadow: `0 2px 8px ${anafisTheme.colors.sidebar.primary}26`,
  '&:hover': {
    background: `linear-gradient(135deg, ${anafisTheme.colors.sidebar.primary}26 0%, ${anafisTheme.colors.sidebar.primary}14 100%)`,
    borderColor: `${anafisTheme.colors.sidebar.primary}80`,
    transform: 'translateY(-1px)',
    boxShadow: `0 4px 16px ${anafisTheme.colors.sidebar.primary}40`,
    color: anafisColors.primary,
  },
  '&:active': {
    background: `linear-gradient(135deg, ${anafisTheme.colors.sidebar.primary}14 0%, ${anafisTheme.colors.sidebar.primary}08 100%)`,
    transform: 'translateY(0px)',
    boxShadow: `0 2px 8px ${anafisTheme.colors.sidebar.primary}26`,
  },
  '&:focus': {
    outline: 'none',
    boxShadow: `0 0 0 2px ${anafisTheme.colors.sidebar.primary}4D`,
  },
  '& .MuiTouchRipple-root': {
    display: 'none',
  },
} as const;

const MENU_PAPER_STYLES = {
  background: anafisTheme.gradients.toolbar,
  backdropFilter: 'blur(20px)',
  border: `1px solid ${anafisTheme.colors.border.light}`,
  borderRadius: 2,
  boxShadow: anafisTheme.shadows.lg,
  mt: 0.5,
  minWidth: '180px',
  '& .MuiMenuItem-root': {
    fontSize: '0.9rem',
    py: 1.5,
    px: 2,
    borderRadius: 1,
    mx: 0.5,
    my: 0.25,
    transition: anafisTheme.transitions.hover,
    '&:hover': {
      backgroundColor: `${anafisTheme.colors.sidebar.primary}1A`,
      color: anafisColors.primary,
      transform: 'translateX(2px)',
    },
  },
} as const;

// Tab button configurations - using unified theme colors
const TAB_BUTTONS_CONFIG = [
  {
    label: 'Spreadsheet',
    type: 'spreadsheet',
    hoverColor: anafisTheme.colors.tabs.spreadsheet.light,
    hoverBackgroundColor: `${anafisTheme.colors.tabs.spreadsheet.main}1F`,
    hoverBorderColor: `${anafisTheme.colors.tabs.spreadsheet.main}33`,
    hoverBoxShadowColor: `${anafisTheme.colors.tabs.spreadsheet.main}4D`,
  },
  {
    label: 'Fitting',
    type: 'fitting',
    hoverColor: anafisTheme.colors.tabs.fitting.light,
    hoverBackgroundColor: `${anafisTheme.colors.tabs.fitting.main}1F`,
    hoverBorderColor: `${anafisTheme.colors.tabs.fitting.main}33`,
    hoverBoxShadowColor: `${anafisTheme.colors.tabs.fitting.main}4D`,
  },
  {
    label: 'Solver',
    type: 'solver',
    hoverColor: anafisTheme.colors.tabs.solver.light,
    hoverBackgroundColor: `${anafisTheme.colors.tabs.solver.main}1F`,
    hoverBorderColor: `${anafisTheme.colors.tabs.solver.main}33`,
    hoverBoxShadowColor: `${anafisTheme.colors.tabs.solver.main}4D`,
  },
  {
    label: 'Monte Carlo',
    type: 'montecarlo',
    hoverColor: anafisTheme.colors.tabs.montecarlo.light,
    hoverBackgroundColor: `${anafisTheme.colors.tabs.montecarlo.main}1F`,
    hoverBorderColor: `${anafisTheme.colors.tabs.montecarlo.main}33`,
    hoverBoxShadowColor: `${anafisTheme.colors.tabs.montecarlo.main}4D`,
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
