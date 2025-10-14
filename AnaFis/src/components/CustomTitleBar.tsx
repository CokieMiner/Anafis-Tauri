import React, { useState, useEffect } from 'react';
import { Box, Typography, IconButton, useTheme } from '@mui/material';
import { Close, Minimize, CropSquare, Reply } from '@mui/icons-material';
import { Home, TableChart, TrendingUp, Calculate, Casino } from '@mui/icons-material';
import { getCurrentWindow } from '@tauri-apps/api/window';

interface CustomTitleBarProps {
  title?: string;
  isDetachedTabWindow?: boolean;
  onReattach?: () => void;
}

const CustomTitleBar: React.FC<CustomTitleBarProps> = ({ title = 'AnaFis', isDetachedTabWindow = false, onReattach }) => {
  const theme = useTheme();
  const [isMaximized, setIsMaximized] = useState(false);
  const [windowReady, setWindowReady] = useState(false);
  const [isTauri, setIsTauri] = useState(false);

  // Get tab icon for detached windows
  const getTabIcon = () => {
    if (!isDetachedTabWindow) return null;

    const urlParams = new URLSearchParams(window.location.search);
    const tabType = urlParams.get('tabType');

  const iconSx = { fontSize: '1.05rem', mr: 0.5, color: undefined, verticalAlign: 'middle', display: 'inline-flex', lineHeight: 1, transform: 'translateY(2px)' } as React.CSSProperties;
    switch (tabType) {
      case 'home':
        return <Home sx={{ ...iconSx, color: '#ba68c8' }} />;
      case 'spreadsheet':
        return <TableChart sx={{ ...iconSx, color: '#64b5f6' }} />;
      case 'fitting':
        return <TrendingUp sx={{ ...iconSx, color: '#ffb74d' }} />;
      case 'solver':
        return <Calculate sx={{ ...iconSx, color: '#81c784' }} />;
      case 'montecarlo':
        return <Casino sx={{ ...iconSx, color: '#f06292' }} />;
      default:
        return <Home sx={{ ...iconSx, color: '#ba68c8' }} />;
    }
  };

  useEffect(() => {
    const initializeWindow = async () => {
      try {
        // Try to get the current window - this will throw if not in Tauri
        const currentWindow = getCurrentWindow();

        // If we get here, we're in Tauri
        setIsTauri(true);

        // Test if window methods are available
        if (currentWindow && typeof currentWindow.isMaximized === 'function') {
          const maximized = await currentWindow.isMaximized();
          setIsMaximized(maximized);
          setWindowReady(true);
        } else {
          console.error('Window object exists but methods are missing');
          setWindowReady(false);
        }
      } catch (error) {
        console.error('Failed to get current window - not in Tauri environment:', error);
        setIsTauri(false);
        setWindowReady(false);

        // Try alternative detection methods

        // Check for Tauri global
        if (typeof window !== 'undefined') {
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          const win = window as any;
          if (win.__TAURI__) {
            try {
              const tauriWindow = win.__TAURI__.window;
              if (tauriWindow) {
                setIsTauri(true);
                setWindowReady(true);
                return;
              }
            } catch (altError) {
              console.error('Alternative access failed:', altError);
            }
          }
        }
      }
    };

    // Small delay to ensure Tauri is fully loaded
    const timer = setTimeout(initializeWindow, 200);

    return () => clearTimeout(timer);
  }, []);

  const getWindowInstance = () => {
    try {
      // Method 1: Standard Tauri API
      return getCurrentWindow();
    } catch {

      // Method 2: Direct global access
      if (typeof window !== 'undefined') {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        const win = window as any;
        if (win.__TAURI__ && win.__TAURI__.window) {
          return win.__TAURI__.window;
        }
      }

      // Method 3: Check if window object has Tauri methods
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      if (typeof window !== 'undefined' && (window as any).minimize) {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        return window as any;
      }

      throw new Error('No window instance available');
    }
  };

  const handleMinimize = async (event: React.MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();

    if (!isTauri) {
      return;
    }

    if (!windowReady) {
      return;
    }

    try {
      const currentWindow = getWindowInstance();

      if (currentWindow.minimize) {
        await currentWindow.minimize();
      } else {
        console.error('Minimize method not available');
      }
    } catch (error) {
      console.error('Minimize failed:', error);
    }
  };

  const handleMaximize = async (event: React.MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();

    if (!isTauri) {
      return;
    }

    if (!windowReady) {
      return;
    }

    try {
      const currentWindow = getWindowInstance();

      let currentlyMaximized = false;
      if (currentWindow.isMaximized) {
        currentlyMaximized = await currentWindow.isMaximized();
      }

      if (currentlyMaximized) {
        if (currentWindow.unmaximize) {
          await currentWindow.unmaximize();
          setIsMaximized(false);
        }
      } else {
        if (currentWindow.maximize) {
          await currentWindow.maximize();
          setIsMaximized(true);
        }
      }
    } catch (error) {
      console.error('Maximize failed:', error);
    }
  };

  const handleClose = async (event: React.MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();

    if (!isTauri) {
      return;
    }

    if (!windowReady) {
      return;
    }

    try {
      const currentWindow = getWindowInstance();

      if (currentWindow.close) {
        await currentWindow.close();
      } else {
        console.error('Close method not available');
      }
    } catch (error) {
      console.error('Close failed:', error);
    }
  };

  const handleDoubleClick = async () => {
    if (!isTauri || !windowReady) {
      return;
    }

    try {
      const currentWindow = getWindowInstance();

      let maximized = false;
      if (currentWindow.isMaximized) {
        maximized = await currentWindow.isMaximized();
      }

      if (maximized) {
        if (currentWindow.unmaximize) {
          await currentWindow.unmaximize();
          setIsMaximized(false);
        }
      } else {
        if (currentWindow.maximize) {
          await currentWindow.maximize();
          setIsMaximized(true);
        }
      }
    } catch (error) {
      console.error('Failed to toggle maximize on double click:', error);
    }
  };

  return (
    <Box
      onDoubleClick={handleDoubleClick}
      sx={{
        height: '36px',
        background: 'linear-gradient(135deg, #1a1a1a 0%, #2a2a2a 100%)',
        borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        px: 2,
        userSelect: 'none',
        WebkitUserSelect: 'none',
        WebkitAppRegion: 'drag', // Makes the entire bar draggable
        position: 'relative',
        zIndex: 1000,
        boxShadow: '0 1px 3px rgba(0, 0, 0, 0.3)',
      }}
    >
      {/* App Title and Reattach Button */}
      <Box sx={{ display: 'flex', alignItems: 'center', flex: 1, minWidth: 0 }}>
        <Box sx={{ display: 'flex', alignItems: 'center', minWidth: 0, height: '100%' }}>
          <Box sx={{ display: 'flex', alignItems: 'center', height: '100%', mr: 0.75 }}>
            {getTabIcon()}
          </Box>

          <Box sx={{ display: 'flex', alignItems: 'center', height: '100%', minWidth: 0 }}>
            <Typography
              variant="body2"
              sx={{
                color: '#ffffff',
                fontWeight: 600,
                fontSize: '0.875rem',
                lineHeight: '20px',
                display: 'inline-flex',
                alignItems: 'center',
                whiteSpace: 'nowrap',
                overflow: 'hidden',
                textOverflow: 'ellipsis',
                WebkitAppRegion: 'no-drag', // Prevents title from interfering with drag
                opacity: 0.95,
              }}
            >
              {title}
            </Typography>
          </Box>

        </Box>

        {/* Reattach Button - Only show for detached tab windows */}
        {isDetachedTabWindow && onReattach && (
          <IconButton
            onClick={onReattach}
            sx={{
              ml: 1,
              width: '24px',
              height: '24px',
              borderRadius: '4px',
              color: 'rgba(255, 255, 255, 0.7)',
              backgroundColor: `${theme.palette.primary.main}20`,
              border: `1px solid ${theme.palette.primary.main}50`,
              WebkitAppRegion: 'no-drag',
              transition: 'all 0.2s ease-in-out',
              '&:hover': {
                backgroundColor: `${theme.palette.primary.main}30`,
                color: theme.palette.primary.main,
                transform: 'scale(1.05)',
                borderColor: `${theme.palette.primary.main}70`,
              },
            }}
            title="Reattach tab to main window"
          >
            <Reply sx={{ fontSize: '14px' }} />
          </IconButton>
        )}

        {/* Status Indicator */}
        <Box
          sx={{
            ml: 1,
            width: '8px',
            height: '8px',
            borderRadius: '50%',
            backgroundColor: isTauri && windowReady ? theme.palette.success.main : isTauri ? theme.palette.warning.main : theme.palette.error.main,
            boxShadow: '0 0 4px rgba(0, 0, 0, 0.3)',
            WebkitAppRegion: 'no-drag',
          }}
          title={isTauri && windowReady ? 'Window controls ready' : isTauri ? 'Initializing...' : 'Not in Tauri environment'}
        />
      </Box>

      {/* Window Controls */}
      <Box
        sx={{
          display: 'flex',
          alignItems: 'center',
          gap: 0,
          WebkitAppRegion: 'no-drag', // Prevents controls from being draggable
        }}
      >
        {/* Minimize Button */}
        <IconButton
          onClick={handleMinimize}
          disabled={!isTauri || !windowReady}
          sx={{
            width: '32px',
            height: '32px',
            borderRadius: 0,
            color: (!isTauri || !windowReady) ? 'rgba(255, 255, 255, 0.3)' : 'rgba(255, 255, 255, 0.8)',
            backgroundColor: 'transparent',
            border: 'none',
            outline: 'none',
            boxShadow: 'none',
            transition: 'all 0.2s ease-in-out',
            '&:hover': {
              backgroundColor: (!isTauri || !windowReady) ? 'transparent' : `${theme.palette.success.main} !important`,
              color: (!isTauri || !windowReady) ? 'rgba(255, 255, 255, 0.3)' : '#ffffff',
              transform: (!isTauri || !windowReady) ? 'none' : 'scale(1.1)',
              boxShadow: (!isTauri || !windowReady) ? 'none' : '0 2px 8px rgba(76, 175, 80, 0.4)',
              outline: 'none !important',
              border: 'none !important',
            },
            '&:active': {
              backgroundColor: (!isTauri || !windowReady) ? 'transparent' : '#388e3c !important',
              transform: (!isTauri || !windowReady) ? 'none' : 'scale(0.95)',
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
            '&.Mui-disabled': {
              color: 'rgba(255, 255, 255, 0.3)',
            },
          }}
        >
          <Minimize sx={{ fontSize: '16px' }} />
        </IconButton>

        {/* Maximize/Restore Button */}
        <IconButton
          onClick={handleMaximize}
          disabled={!isTauri || !windowReady}
          sx={{
            width: '32px',
            height: '32px',
            borderRadius: 0,
            color: (!isTauri || !windowReady) ? 'rgba(255, 255, 255, 0.3)' : 'rgba(255, 255, 255, 0.8)',
            backgroundColor: 'transparent',
            border: 'none',
            outline: 'none',
            boxShadow: 'none',
            transition: 'all 0.2s ease-in-out',
            '&:hover': {
              backgroundColor: (!isTauri || !windowReady) ? 'transparent' : `${theme.palette.secondary.main} !important`,
              color: (!isTauri || !windowReady) ? 'rgba(255, 255, 255, 0.3)' : '#ffffff',
              transform: (!isTauri || !windowReady) ? 'none' : 'scale(1.1)',
              boxShadow: (!isTauri || !windowReady) ? 'none' : `0 2px 8px ${theme.palette.secondary.main}40`,
              outline: 'none !important',
              border: 'none !important',
            },
            '&:active': {
              backgroundColor: (!isTauri || !windowReady) ? 'transparent' : `${theme.palette.secondary.main} !important`,
              transform: (!isTauri || !windowReady) ? 'none' : 'scale(0.95)',
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
            '&.Mui-disabled': {
              color: 'rgba(255, 255, 255, 0.3)',
            },
          }}
        >
          <CropSquare
            sx={{
              fontSize: '14px',
              transform: isMaximized ? 'rotate(180deg)' : 'rotate(0deg)',
              transition: 'transform 0.2s ease-in-out',
            }}
          />
        </IconButton>

        {/* Close Button */}
        <IconButton
          onClick={handleClose}
          disabled={!isTauri || !windowReady}
          sx={{
            width: '32px',
            height: '32px',
            borderRadius: 0,
            color: (!isTauri || !windowReady) ? 'rgba(255, 255, 255, 0.3)' : 'rgba(255, 255, 255, 0.8)',
            backgroundColor: 'transparent',
            border: 'none',
            outline: 'none',
            boxShadow: 'none',
            transition: 'all 0.2s ease-in-out',
            '&:hover': {
              backgroundColor: (!isTauri || !windowReady) ? 'transparent' : `${theme.palette.error.main} !important`,
              color: (!isTauri || !windowReady) ? 'rgba(255, 255, 255, 0.3)' : '#ffffff',
              transform: (!isTauri || !windowReady) ? 'none' : 'scale(1.1)',
              boxShadow: (!isTauri || !windowReady) ? 'none' : '0 2px 8px rgba(244, 67, 54, 0.4)',
              outline: 'none !important',
              border: 'none !important',
            },
            '&:active': {
              backgroundColor: (!isTauri || !windowReady) ? 'transparent' : `${theme.palette.error.dark} !important`,
              transform: (!isTauri || !windowReady) ? 'none' : 'scale(0.95)',
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
            '&.Mui-disabled': {
              color: 'rgba(255, 255, 255, 0.3)',
            },
          }}
        >
          <Close sx={{ fontSize: '16px' }} />
        </IconButton>
      </Box>
    </Box>
  );
};

export default CustomTitleBar;
