import React, { useState, useEffect } from 'react';
import { Box, Typography, IconButton, useTheme } from '@mui/material';
import { Close, Minimize, CropSquare } from '@mui/icons-material';
import { getCurrentWindow } from '@tauri-apps/api/window';

// Define Tauri window interface
interface TauriWindow {
  minimize(): Promise<void>;
  maximize(): Promise<void>;
  unmaximize(): Promise<void>;
  close(): Promise<void>;
  isMaximized(): Promise<boolean>;
}

// Type guard to safely check if an object is a TauriWindow
function isTauriWindow(obj: unknown): obj is TauriWindow {
  const candidate = obj as Record<string, unknown>;
  return (
    typeof obj === 'object' &&
    obj !== null &&
    typeof candidate.minimize === 'function' &&
    typeof candidate.maximize === 'function' &&
    typeof candidate.unmaximize === 'function' &&
    typeof candidate.close === 'function' &&
    typeof candidate.isMaximized === 'function'
  );
}

// Define global Tauri interface
interface TauriGlobal {
  __TAURI__?: {
    window?: TauriWindow;
  };
}

const CustomTitleBar: React.FC<{ title: string }> = ({ title }) => {
  const theme = useTheme();
  const [isMaximized, setIsMaximized] = useState(false);
  const [windowReady, setWindowReady] = useState(false);
  const [isTauri, setIsTauri] = useState(false);

  // Get tab icon for detached windows
  const getTabIcon = () => {
    return null;
  };

  useEffect(() => {
    const initializeWindow = async () => {
      try {
        // Try to get the current window - this will throw if not in Tauri
        const currentWindow = getCurrentWindow();

        // If we get here, we're in Tauri
        setIsTauri(true);

        // Test if window methods are available
        if (typeof currentWindow.isMaximized === 'function') {
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
          const win = window as Window & TauriGlobal;
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

    initializeWindow().catch((err) => {
      console.error('Initialization error:', err);
    });
  }, []);

  const getWindowInstance = (): TauriWindow => {
    try {
      // Method 1: Standard Tauri API
      return getCurrentWindow();
    } catch {

      // Method 2: Direct global access
      if (typeof window !== 'undefined') {
        const win = window as Window & TauriGlobal;
        if (win.__TAURI__?.window) {
          return win.__TAURI__.window;
        }
      }

      // Method 3: Check if window object has all required Tauri methods
      if (typeof window !== 'undefined' && isTauriWindow(window)) {
        return window;
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
      await currentWindow.minimize();
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

      const currentlyMaximized = await currentWindow.isMaximized();

      if (currentlyMaximized) {
        await currentWindow.unmaximize();
        setIsMaximized(false);
      } else {
        await currentWindow.maximize();
        setIsMaximized(true);
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
      await currentWindow.close();
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

      const maximized = await currentWindow.isMaximized();

      if (maximized) {
        await currentWindow.unmaximize();
        setIsMaximized(false);
      } else {
        await currentWindow.maximize();
        setIsMaximized(true);
      }
    } catch (error) {
      console.error('Failed to toggle maximize on double click:', error);
    }
  };

  return (
    <div
      data-tauri-drag-region
      onDoubleClick={() => { handleDoubleClick().catch(console.error); return; }}
      style={{
        height: '36px',
        background: 'linear-gradient(135deg, #1a1a1a 0%, #2a2a2a 100%)',
        borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        paddingLeft: '16px',
        paddingRight: '16px',
        userSelect: 'none',
        WebkitUserSelect: 'none',
        position: 'relative',
        zIndex: 1000,
        boxShadow: '0 1px 3px rgba(0, 0, 0, 0.3)',
        cursor: 'default',
      }}
    >
      {/* App Title and Reattach Button */}
      <div
        data-tauri-drag-region
        style={{ display: 'flex', alignItems: 'center', flex: 1, minWidth: 0 }}
      >
        <div
          data-tauri-drag-region
          style={{ display: 'flex', alignItems: 'center', minWidth: 0, height: '100%' }}
        >
          <div
            data-tauri-drag-region
            style={{ display: 'flex', alignItems: 'center', height: '100%', marginRight: '6px' }}
          >
            {getTabIcon()}
          </div>
          <div
            data-tauri-drag-region
            style={{ display: 'flex', alignItems: 'center', height: '100%', minWidth: 0 }}
          >
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
                opacity: 0.95,
                pointerEvents: 'none',
              }}>
              {title}
            </Typography>
          </div>
        </div>

        {/* Status Indicator */}
        <Box
          sx={{
            ml: 1,
            width: '8px',
            height: '8px',
            borderRadius: '50%',
            backgroundColor: isTauri && windowReady ? theme.palette.success.main : isTauri ? theme.palette.warning.main : theme.palette.error.main,
            boxShadow: '0 0 4px rgba(0, 0, 0, 0.3)',
          }}
          title={isTauri && windowReady ? 'Window controls ready' : isTauri ? 'Initializing...' : 'Not in Tauri environment'}
        />
      </div>

      {/* Window Controls */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 0,
        }}
      >
        {/* Minimize Button */}
        <IconButton
          onClick={(event) => void handleMinimize(event)}
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
          onClick={(event) => void handleMaximize(event)}
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
          onClick={(event) => void handleClose(event)}
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
      </div>
    </div>
  );
};

export default CustomTitleBar;
