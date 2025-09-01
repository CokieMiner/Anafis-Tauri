import React, { useState } from 'react';
import { createRoot } from 'react-dom/client';
import { ThemeProvider, createTheme } from '@mui/material';
import CssBaseline from '@mui/material/CssBaseline';
import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import IconButton from '@mui/material/IconButton';
import Tabs from '@mui/material/Tabs';
import Tab from '@mui/material/Tab';
import FormControl from '@mui/material/FormControl';
import InputLabel from '@mui/material/InputLabel';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import Switch from '@mui/material/Switch';
import FormControlLabel from '@mui/material/FormControlLabel';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';
import { Close } from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';

// Dark theme for the settings window
const theme = createTheme({
  palette: {
    mode: 'dark',
    primary: {
      main: '#9c27b0',
      light: '#ba68c8',
      dark: '#7b1fa2',
    },
    background: {
      default: '#0a0a0a',
      paper: '#111111',
    },
    text: {
      primary: '#ffffff',
      secondary: 'rgba(255, 255, 255, 0.7)',
    },
  },
  typography: {
    fontFamily: '"Inter", "Roboto", "Helvetica", "Arial", sans-serif',
  },
  components: {
    MuiButton: {
      styleOverrides: {
        root: {
          borderRadius: 8,
          textTransform: 'none',
        },
      },
    },
    MuiSwitch: {
      styleOverrides: {
        root: {
          '& .MuiSwitch-switchBase.Mui-checked': {
            color: '#9c27b0 !important',
          },
          '& .MuiSwitch-switchBase.Mui-checked + .MuiSwitch-track': {
            backgroundColor: '#9c27b0 !important',
          },
          '& .MuiSwitch-switchBase': {
            color: 'rgba(255, 255, 255, 0.7)',
          },
          '& .MuiSwitch-track': {
            backgroundColor: 'rgba(255, 255, 255, 0.3)',
          },
        },
      },
    },
    MuiTab: {
      styleOverrides: {
        root: {
          color: 'rgba(255, 255, 255, 0.7) !important',
          '&.Mui-selected': {
            color: '#9c27b0 !important',
          },
          '&:hover': {
            color: 'rgba(255, 255, 255, 0.9) !important',
          },
          '&.Mui-selected:hover': {
            color: '#9c27b0 !important',
          },
        },
      },
    },
    MuiTypography: {
      styleOverrides: {
        root: {
          color: '#ffffff !important',
        },
      },
    },
  },
});

interface TabPanelProps {
  children?: React.ReactNode;
  index: number;
  value: number;
}

function TabPanel(props: TabPanelProps) {
  const { children, value, index, ...other } = props;

  return (
    <div
      role="tabpanel"
      id={`settings-tabpanel-${index}`}
      aria-labelledby={`settings-tab-${index}`}
      style={{ display: value === index ? 'block' : 'none' }}
      {...other}
    >
      <Box sx={{ p: 3 }}>
        {children}
      </Box>
    </div>
  );
}

function SettingsWindow() {
  const [tabValue, setTabValue] = useState(0);
  const [language, setLanguage] = useState('en');
  const [autoUpdate, setAutoUpdate] = useState(true);
  const [updateFrequency, setUpdateFrequency] = useState('weekly');

  const handleTabChange = (_event: React.SyntheticEvent, newValue: number) => {
    setTabValue(newValue);
  };

  const handleClose = async () => {
    try {
      await invoke('close_settings_window');
    } catch (error) {
      console.error('Failed to close settings window:', error);
    }
  };

  const handleSave = () => {
    // Save settings logic would go here
    handleClose();
  };

  const handleReset = () => {
    setLanguage('en');
    setAutoUpdate(true);
    setUpdateFrequency('weekly');
  };

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Box
        sx={{
          width: '650px',
          height: '700px',
          backgroundColor: 'background.default',
          display: 'flex',
          flexDirection: 'column',
          overflow: 'hidden',
        }}
      >
        {/* Custom Title Bar */}
        <Box
          sx={{
            height: '40px',
            background: 'linear-gradient(135deg, #1a1a1a 0%, #2a2a2a 100%)',
            borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            px: 2,
            WebkitAppRegion: 'drag',
            flexShrink: 0,
            isolation: 'isolate',
            '&:hover': {
              background: 'linear-gradient(135deg, #1a1a1a 0%, #2a2a2a 100%)',
            },
          }}
        >
          <Typography
            variant="body2"
            sx={{
              color: '#ffffff !important',
              fontWeight: 600,
              fontSize: '0.875rem',
              WebkitAppRegion: 'no-drag',
            }}
          >
            AnaFis Settings
          </Typography>

          <Box sx={{ isolation: 'isolate' }}>
            <IconButton
              onClick={handleClose}
              sx={{
                width: '32px',
                height: '32px',
                borderRadius: 0,
                color: 'rgba(255, 255, 255, 0.8)',
                backgroundColor: 'transparent',
                border: 'none',
                outline: 'none',
                boxShadow: 'none',
                transition: 'all 0.2s ease-in-out',
                WebkitAppRegion: 'no-drag',
                '&:hover': {
                  backgroundColor: '#f44336 !important',
                  color: '#ffffff',
                  transform: 'scale(1.1)',
                  boxShadow: '0 2px 8px rgba(244, 67, 54, 0.4)',
                  outline: 'none !important',
                  border: 'none !important',
                },
                '&:active': {
                  backgroundColor: '#d32f2f !important',
                  transform: 'scale(0.95)',
                  outline: 'none !important',
                  border: 'none !important',
                  boxShadow: 'none !important',
                },
                '&:focus': {
                  outline: 'none !important',
                  border: 'none !important',
                  boxShadow: 'none !important',
                },
                '&.Mui-focusVisible': {
                  outline: 'none !important',
                  border: 'none !important',
                  boxShadow: 'none !important',
                },
              }}
            >
              <Close sx={{ fontSize: '16px' }} />
            </IconButton>
          </Box>
        </Box>

        {/* Main Content */}
        <Box sx={{ flex: 1, display: 'flex', flexDirection: 'column', minHeight: 0 }}>
          {/* Tabs */}
          <Box sx={{ borderBottom: 1, borderColor: 'divider', flexShrink: 0 }}>
            <Tabs value={tabValue} onChange={handleTabChange} aria-label="settings tabs">
              <Tab label="General" />
              <Tab label="Updates" />
            </Tabs>
          </Box>

          {/* Tab Content */}
          <Box sx={{ flex: 1, overflow: 'auto', minHeight: 0 }}>
            {/* General Tab */}
            <TabPanel value={tabValue} index={0}>
              <Typography variant="h6" gutterBottom>
                General Settings
              </Typography>

              <Box sx={{ mt: 2 }}>
                <FormControl fullWidth sx={{ mb: 3 }}>
                  <InputLabel sx={{
                    '&.Mui-focused': {
                      color: '#9c27b0 !important',
                    }
                  }}>Language</InputLabel>
                  <Select
                    value={language}
                    label="Language"
                    onChange={(e) => setLanguage(e.target.value)}
                    sx={{
                      '& .MuiOutlinedInput-notchedOutline': {
                        borderColor: 'rgba(255, 255, 255, 0.5)',
                      },
                      '&:hover .MuiOutlinedInput-notchedOutline': {
                        borderColor: 'rgba(255, 255, 255, 0.7)',
                      },
                      '&.Mui-focused .MuiOutlinedInput-notchedOutline': {
                        borderColor: '#9c27b0 !important',
                      },
                    }}
                  >
                    <MenuItem value="en">English</MenuItem>
                    <MenuItem value="pt">Português</MenuItem>
                    <MenuItem value="es">Español</MenuItem>
                    <MenuItem value="fr">Français</MenuItem>
                    <MenuItem value="de">Deutsch</MenuItem>
                  </Select>
                </FormControl>

                <Typography variant="body2" color="text.secondary">
                  Choose your preferred language for the application interface.
                </Typography>
              </Box>
            </TabPanel>

            {/* Updates Tab */}
            <TabPanel value={tabValue} index={1}>
              <Typography variant="h6" gutterBottom>
                Update Settings
              </Typography>

              <Box sx={{ mt: 2 }}>
                <Box sx={{
                  mb: 2,
                  isolation: 'isolate',
                }}>
                  <FormControlLabel
                    control={
                      <Switch
                        checked={autoUpdate}
                        onChange={(e) => setAutoUpdate(e.target.checked)}
                        sx={{
                          // Additional specific overrides to ensure consistency
                          '& .MuiSwitch-switchBase.Mui-checked': {
                            color: '#9c27b0 !important',
                          },
                          '& .MuiSwitch-switchBase.Mui-checked + .MuiSwitch-track': {
                            backgroundColor: '#9c27b0 !important',
                          },
                        }}
                      />
                    }
                    label="Enable automatic updates"
                  />
                </Box>

                <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
                  When enabled, AnaFis will automatically check for and download updates.
                </Typography>

                <FormControl fullWidth sx={{ mb: 3 }} disabled={!autoUpdate}>
                  <InputLabel sx={{
                    '&.Mui-focused': {
                      color: '#9c27b0 !important',
                    }
                  }}>Check for updates</InputLabel>
                  <Select
                    value={updateFrequency}
                    label="Check for updates"
                    onChange={(e) => setUpdateFrequency(e.target.value)}
                    sx={{
                      '& .MuiOutlinedInput-notchedOutline': {
                        borderColor: 'rgba(255, 255, 255, 0.5)',
                      },
                      '&:hover .MuiOutlinedInput-notchedOutline': {
                        borderColor: 'rgba(255, 255, 255, 0.7)',
                      },
                      '&.Mui-focused .MuiOutlinedInput-notchedOutline': {
                        borderColor: '#9c27b0 !important',
                      },
                    }}
                  >
                    <MenuItem value="daily">Daily</MenuItem>
                    <MenuItem value="weekly">Weekly</MenuItem>
                    <MenuItem value="monthly">Monthly</MenuItem>
                    <MenuItem value="manual">Manual only</MenuItem>
                  </Select>
                </FormControl>

                <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
                  How often should AnaFis check for available updates?
                </Typography>

                <Divider sx={{ my: 2 }} />

                <Box sx={{ display: 'flex', gap: 1, flexWrap: 'wrap' }}>
                  <Button variant="outlined" size="small">
                    Check for Updates Now
                  </Button>
                  <Button variant="outlined" size="small">
                    View Update History
                  </Button>
                </Box>
              </Box>
            </TabPanel>
          </Box>

          {/* Action Buttons */}
          <Box sx={{
            display: 'flex',
            justifyContent: 'flex-end',
            gap: 1,
            p: 2,
            borderTop: 1,
            borderColor: 'divider',
            flexShrink: 0,
            minHeight: 'fit-content',
          }}>
            <Button onClick={handleReset} color="secondary">
              Reset to Defaults
            </Button>
            <Button onClick={handleClose} color="secondary">
              Cancel
            </Button>
            <Button onClick={handleSave} variant="contained">
              Save Settings
            </Button>
          </Box>
        </Box>
      </Box>
    </ThemeProvider>
  );
}

// Auto-render immediately when this module loads

const renderSettingsWindow = () => {
  const container = document.getElementById('root');
  if (container) {
    try {
      const root = createRoot(container);
      root.render(<SettingsWindow />);
    } catch (error) {
      console.error('SettingsWindow: Error rendering:', error);
    }
  } else {
    console.error('SettingsWindow: Root container not found');
  }
};

// Try to render immediately
if (document.readyState === 'complete') {
  renderSettingsWindow();
} else {
  // Wait for DOM to be ready
  window.addEventListener('load', renderSettingsWindow);
}

export default SettingsWindow;
