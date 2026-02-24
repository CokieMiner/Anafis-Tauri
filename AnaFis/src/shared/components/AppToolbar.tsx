// Extracted and optimized app toolbar component

import { AppBar, Box, IconButton, Toolbar } from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import React, { useMemo, useRef } from 'react';
import { CalculateIcon, SettingsIcon, StorageIcon } from '@/icons';
import TabButton from '@/shared/components/TabButton';
import { anafisTheme } from '@/shared/theme/unifiedTheme';

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
    // Incremental counter for unique tab IDs
    const tabIdCounter = useRef<number>(0);

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

          {/* Spacer */}
          <Box sx={{ flexGrow: 1 }} />

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
