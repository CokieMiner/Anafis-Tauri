import React, { useState, useEffect } from 'react';
import { Box, Typography, IconButton } from '@mui/material';
import { Close, Minimize, CropSquare } from '@mui/icons-material';
import { getCurrentWindow } from '@tauri-apps/api/window';

interface CustomTitleBarProps {
  title?: string;
}

const CustomTitleBar: React.FC<CustomTitleBarProps> = ({ title = 'AnaFis' }) => {
  const [isMaximized, setIsMaximized] = useState(false);
  const [windowReady, setWindowReady] = useState(false);
  const [isTauri, setIsTauri] = useState(false);

  useEffect(() => {
    const initializeWindow = async () => {
      console.log('Starting window initialization...');

      try {
        // Try to get the current window - this will throw if not in Tauri
        console.log('Attempting to get current window...');
        const currentWindow = getCurrentWindow();
        console.log('Successfully got window object:', currentWindow);

        // If we get here, we're in Tauri
        setIsTauri(true);
        console.log('Confirmed: Running in Tauri environment');

        // Test if window methods are available
        if (currentWindow && typeof currentWindow.isMaximized === 'function') {
          console.log('Testing window.isMaximized()...');
          const maximized = await currentWindow.isMaximized();
          console.log('Window maximized state:', maximized);
          setIsMaximized(maximized);
          setWindowReady(true);
          console.log('Window initialization completed successfully');
        } else {
          console.error('Window object exists but methods are missing');
          setWindowReady(false);
        }
      } catch (error) {
        console.error('Failed to get current window - not in Tauri environment:', error);
        setIsTauri(false);
        setWindowReady(false);

        // Try alternative detection methods
        console.log('Trying alternative Tauri detection...');

        // Check for Tauri global
        if (typeof window !== 'undefined') {
          const win = window as any;
          if (win.__TAURI__) {
            console.log('Found __TAURI__ global, attempting direct access...');
            try {
              const tauriWindow = win.__TAURI__.window;
              if (tauriWindow) {
                console.log('Found Tauri window via global, setting up controls...');
                setIsTauri(true);
                setWindowReady(true);
                console.log('Alternative initialization successful');
                return;
              }
            } catch (altError) {
              console.error('Alternative access failed:', altError);
            }
          }
        }

        console.log('All Tauri detection methods failed - running in web mode');
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
    } catch (error) {
      console.log('Standard API failed, trying alternatives...');

      // Method 2: Direct global access
      if (typeof window !== 'undefined') {
        const win = window as any;
        if (win.__TAURI__ && win.__TAURI__.window) {
          console.log('Using direct global access');
          return win.__TAURI__.window;
        }
      }

      // Method 3: Check if window object has Tauri methods
      if (typeof window !== 'undefined' && (window as any).minimize) {
        console.log('Using window object methods');
        return window as any;
      }

      throw new Error('No window instance available');
    }
  };

  const handleMinimize = async (event: React.MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();

    if (!isTauri) {
      console.log('Not in Tauri environment, cannot minimize');
      return;
    }

    if (!windowReady) {
      console.log('Window not ready yet, cannot minimize');
      return;
    }

    console.log('Minimize button clicked');

    try {
      const currentWindow = getWindowInstance();
      console.log('Got window instance, calling minimize...');

      if (currentWindow.minimize) {
        await currentWindow.minimize();
        console.log('Minimize successful');
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
      console.log('Not in Tauri environment, cannot maximize');
      return;
    }

    if (!windowReady) {
      console.log('Window not ready yet, cannot maximize');
      return;
    }

    console.log('Maximize button clicked');

    try {
      const currentWindow = getWindowInstance();
      console.log('Got window instance, checking maximize state...');

      let currentlyMaximized = false;
      if (currentWindow.isMaximized) {
        currentlyMaximized = await currentWindow.isMaximized();
      }

      console.log('Current maximized state:', currentlyMaximized);

      if (currentlyMaximized) {
        if (currentWindow.unmaximize) {
          await currentWindow.unmaximize();
          setIsMaximized(false);
          console.log('Unmaximize successful');
        }
      } else {
        if (currentWindow.maximize) {
          await currentWindow.maximize();
          setIsMaximized(true);
          console.log('Maximize successful');
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
      console.log('Not in Tauri environment, cannot close');
      return;
    }

    if (!windowReady) {
      console.log('Window not ready yet, cannot close');
      return;
    }

    console.log('Close button clicked');

    try {
      const currentWindow = getWindowInstance();
      console.log('Got window instance, calling close...');

      if (currentWindow.close) {
        await currentWindow.close();
        console.log('Close successful');
      } else {
        console.error('Close method not available');
      }
    } catch (error) {
      console.error('Close failed:', error);
    }
  };

  const handleDoubleClick = async () => {
    if (!isTauri || !windowReady) {
      console.log('Cannot toggle maximize: Tauri not ready');
      return;
    }

    try {
      console.log('Double-click detected, toggling maximize...');
      const currentWindow = getWindowInstance();

      let maximized = false;
      if (currentWindow.isMaximized) {
        maximized = await currentWindow.isMaximized();
      }

      if (maximized) {
        if (currentWindow.unmaximize) {
          await currentWindow.unmaximize();
          setIsMaximized(false);
          console.log('Window unmaximized via double-click');
        }
      } else {
        if (currentWindow.maximize) {
          await currentWindow.maximize();
          setIsMaximized(true);
          console.log('Window maximized via double-click');
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
        height: '32px',
        background: 'linear-gradient(135deg, #2a2a2a 0%, #3a3a3a 100%)',
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
      {/* App Title */}
      <Box sx={{ display: 'flex', alignItems: 'center', flex: 1, minWidth: 0 }}>
        <Typography
          variant="body2"
          sx={{
            color: '#ffffff',
            fontWeight: 600,
            fontSize: '0.875rem',
            whiteSpace: 'nowrap',
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            WebkitAppRegion: 'no-drag', // Prevents title from interfering with drag
            opacity: 0.9,
            '&:hover': {
              opacity: 1,
            },
          }}
        >
          {title}
        </Typography>
        {/* Status Indicator */}
        <Box
          sx={{
            ml: 1,
            width: '8px',
            height: '8px',
            borderRadius: '50%',
            backgroundColor: isTauri && windowReady ? '#4caf50' : isTauri ? '#ff9800' : '#f44336',
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
              backgroundColor: (!isTauri || !windowReady) ? 'transparent' : '#4caf50 !important',
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
              backgroundColor: (!isTauri || !windowReady) ? 'transparent' : '#2196f3 !important',
              color: (!isTauri || !windowReady) ? 'rgba(255, 255, 255, 0.3)' : '#ffffff',
              transform: (!isTauri || !windowReady) ? 'none' : 'scale(1.1)',
              boxShadow: (!isTauri || !windowReady) ? 'none' : '0 2px 8px rgba(33, 150, 243, 0.4)',
              outline: 'none !important',
              border: 'none !important',
            },
            '&:active': {
              backgroundColor: (!isTauri || !windowReady) ? 'transparent' : '#1976d2 !important',
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
              backgroundColor: (!isTauri || !windowReady) ? 'transparent' : '#f44336 !important',
              color: (!isTauri || !windowReady) ? 'rgba(255, 255, 255, 0.3)' : '#ffffff',
              transform: (!isTauri || !windowReady) ? 'none' : 'scale(1.1)',
              boxShadow: (!isTauri || !windowReady) ? 'none' : '0 2px 8px rgba(244, 67, 54, 0.4)',
              outline: 'none !important',
              border: 'none !important',
            },
            '&:active': {
              backgroundColor: (!isTauri || !windowReady) ? 'transparent' : '#d32f2f !important',
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
