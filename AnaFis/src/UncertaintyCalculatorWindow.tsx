import { createRoot } from 'react-dom/client';
import { useEffect, useRef } from 'react';
import { ThemeProvider, createTheme } from '@mui/material/styles';
import CssBaseline from '@mui/material/CssBaseline';
import Box from '@mui/material/Box';
import { Typography, IconButton } from '@mui/material';
import { Close } from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import UncertaintyCalculatorDialog from './dialogs/UncertaintyCalculatorDialog';

// Create a dark theme consistent with the main app
const theme = createTheme({
  palette: {
    mode: 'dark',
    primary: {
      main: '#1e1b4b', // Deep dark blue-purple
      light: '#3730a3',
      dark: '#1e1b4b',
    },
    secondary: {
      main: '#7f1d1d', // Deep dark red
      light: '#991b1b',
      dark: '#450a0a',
    },
    background: {
      default: '#0a0a0a', // Pure black background
      paper: '#111111', // Very dark gray for cards
    },
    text: {
      primary: '#ffffff', // Pure white text
      secondary: '#ffffff', // Pure white secondary text
    },
    success: {
      main: '#166534', // Dark green
    },
    warning: {
      main: '#92400e', // Dark orange
    },
    error: {
      main: '#7f1d1d', // Deep dark red
    },
  },
  typography: {
    fontFamily: '"Inter", "Roboto", "Helvetica", "Arial", sans-serif',
    h1: {
      fontWeight: 700,
      fontSize: '2.5rem',
      letterSpacing: '-0.02em',
    },
    h2: {
      fontWeight: 600,
      fontSize: '2rem',
      letterSpacing: '-0.01em',
    },
    h5: {
      fontWeight: 600,
      fontSize: '1.25rem',
    },
    button: {
      textTransform: 'none',
      fontWeight: 500,
    },
  },
  shape: {
    borderRadius: 12,
  },
  components: {
    MuiAppBar: {
      styleOverrides: {
        root: {
          backgroundColor: '#111111',
          backgroundImage: 'linear-gradient(135deg, rgba(30, 27, 75, 0.15) 0%, rgba(79, 29, 29, 0.15) 100%)',
          backdropFilter: 'blur(10px)',
          borderBottom: '1px solid rgba(255, 255, 255, 0.08)',
        },
      },
    },
    MuiPaper: {
      styleOverrides: {
        root: {
          backgroundColor: '#111111',
          backgroundImage: 'linear-gradient(135deg, rgba(255, 255, 255, 0.03) 0%, rgba(255, 255, 255, 0.01) 100%)',
          border: '1px solid rgba(255, 255, 255, 0.08)',
          transition: 'all 0.3s ease-in-out',
          '&:hover': {
            boxShadow: '0 8px 32px rgba(30, 27, 75, 0.15)',
            border: '1px solid rgba(30, 27, 75, 0.25)',
          },
        },
      },
    },
    MuiButton: {
      styleOverrides: {
        root: {
          borderRadius: 8,
          padding: '8px 16px',
          transition: 'all 0.2s ease-in-out',
          '&:hover': {
            transform: 'translateY(-1px)',
            boxShadow: '0 4px 12px rgba(30, 27, 75, 0.2)',
          },
        },
        contained: {
          background: 'linear-gradient(135deg, #1e1b4b 0%, #3730a3 100%)',
          '&:hover': {
            background: 'linear-gradient(135deg, #3730a3 0%, #1e1b4b 100%)',
          },
        },
      },
    },
    MuiCssBaseline: {
      styleOverrides: {
        body: {
          scrollbarWidth: 'thin',
          '&::-webkit-scrollbar': {
            width: '8px',
          },
          '&::-webkit-scrollbar-track': {
            background: '#111111',
          },
          '&::-webkit-scrollbar-thumb': {
            background: '#5a5a5a',
            borderRadius: '4px',
          },
          '&::-webkit-scrollbar-thumb:hover': {
            background: '#7a7a7a',
          },
        },
      },
    },
  },
});

function UncertaintyCalculatorWindow() {
  const contentRef = useRef<HTMLDivElement>(null);
  const resizeTimeoutRef = useRef<number | null>(null);
  const followUpTimeoutRef = useRef<number | null>(null);
  const overflowPrevRef = useRef<string | null>(null);
  const lastSizeRef = useRef<{width: number, height: number} | null>(null);

  const resizeWindow = async () => {
    if (contentRef.current && resizeTimeoutRef.current === null) {
      const el = contentRef.current;

  // Use scroll/offset sizes to ensure full content is visible
  const measuredHeight = Math.max(el.offsetHeight || 0, el.scrollHeight || 0);

      // Add padding so scrollbars never appear by accident
  // fixed width must match the container width above
  const width = 504;
  const height = Math.max(Math.ceil(measuredHeight + 20), 450);

      // Check if size actually changed significantly (more than 6px difference) to reduce chatter
      if (lastSizeRef.current &&
          Math.abs(lastSizeRef.current.width - width) < 6 &&
          Math.abs(lastSizeRef.current.height - height) < 6) {
        return; // Size hasn't changed significantly, skip resize
      }

      lastSizeRef.current = { width, height };

      // Debounce the resize to prevent infinite loops
      resizeTimeoutRef.current = window.setTimeout(async () => {
        try {
          await invoke('resize_uncertainty_calculator_window', {
            width: Math.round(width * window.devicePixelRatio),
            height: Math.round(height * window.devicePixelRatio),
          });
        } catch (error) {
          console.error('Failed to resize window:', error);
        } finally {
          resizeTimeoutRef.current = null;
        }
      }, 160);
    }
  };

  // Immediate resize that invokes the backend without debounce.
  // Accepts an extraHeight padding to grow the window more to avoid temporary scrollbars.
  const resizeNow = async (extraHeight = 0) => {
    if (!contentRef.current) return;
  const el = contentRef.current;
  const measuredHeight = Math.max(el.offsetHeight || 0, el.scrollHeight || 0);
  const width = 504; // fixed
    const height = Math.max(Math.ceil(measuredHeight + 20 + extraHeight), 450);

    // Only send if size differs enough
    if (lastSizeRef.current && Math.abs(lastSizeRef.current.width - width) < 6 && Math.abs(lastSizeRef.current.height - height) < 6) {
      return;
    }
    lastSizeRef.current = { width, height };

    try {
      await invoke('resize_uncertainty_calculator_window', {
        width: Math.round(width * window.devicePixelRatio),
        height: Math.round(height * window.devicePixelRatio),
      });
    } catch (error) {
      console.error('Failed to resize window (immediate):', error);
    }
  };

  useEffect(() => {
    // Initial resize after component mounts
    const timer = window.setTimeout(resizeWindow, 500);

    // Immediately hide overflow and perform an immediate oversize resize to avoid transient scrollbars
    try {
      overflowPrevRef.current = document.documentElement.style.overflow || null;
      document.documentElement.style.overflow = 'hidden';
    } catch (e) {
      // ignore
    }

    // Do an immediate larger resize to reduce flicker while the app finishes mounting
    (async () => {
      try {
        await resizeNow(60);
      } catch (e) {
        // ignore
      }
      // schedule a short follow-up to fine-tune and restore overflow
      if (followUpTimeoutRef.current) {
        window.clearTimeout(followUpTimeoutRef.current as number);
        followUpTimeoutRef.current = null;
      }
      followUpTimeoutRef.current = window.setTimeout(() => {
        try {
          if (overflowPrevRef.current !== null) {
            document.documentElement.style.overflow = overflowPrevRef.current;
          } else {
            document.documentElement.style.overflow = '';
          }
        } catch (e) {
          // ignore
        }

        resizeWindow();
        followUpTimeoutRef.current = null;
      }, 300);
    })();

    // Try to disable manual resizing for this native window while this component is mounted.
    // Some environments initialize the window API slightly later, so try a few times.
    let restored = false;
    const tryDisableResizable = async () => {
      try {
        const w = getCurrentWindow();
        if (w && typeof (w as any).setResizable === 'function') {
          try {
            await (w as any).setResizable(false);
            restored = true;
            return true;
          } catch (err) {
            console.warn('Could not set resizable(false) on attempt:', err);
          }
        }
      } catch (err) {
        // ignore
      }
      return false;
    };

    (async () => {
      // Try immediately, then with small backoffs in case the API isn't ready.
      if (await tryDisableResizable()) return;
      setTimeout(async () => { if (await tryDisableResizable()) return; }, 200);
      setTimeout(async () => { await tryDisableResizable(); }, 600);
    })();

    // Resize when window content changes (but less frequently)
    const resizeObserver = new ResizeObserver(() => {
      // Only resize if not already resizing
      if (resizeTimeoutRef.current === null) {
        resizeWindow();
      }
    });

    if (contentRef.current) {
      resizeObserver.observe(contentRef.current);
    }

    // Listen for explicit content-change events fired by dialog so we can resize immediately
    const onContentChange = () => {
      // Cancel pending debounced resize
      if (resizeTimeoutRef.current) {
        window.clearTimeout(resizeTimeoutRef.current as number);
        resizeTimeoutRef.current = null;
      }

      // Temporarily hide document scrollbars to avoid visual flicker while we resize the native window
      try {
        overflowPrevRef.current = document.documentElement.style.overflow || null;
        document.documentElement.style.overflow = 'hidden';
      } catch (e) {
        // ignore any DOM access errors in exotic environments
      }

      // Immediate larger resize to avoid temporary scrollbars (bigger padding)
      resizeNow(40);

      // Clear any previous follow-up and schedule a fine-tune call that will restore overflow
      if (followUpTimeoutRef.current) {
        window.clearTimeout(followUpTimeoutRef.current as number);
        followUpTimeoutRef.current = null;
      }
      followUpTimeoutRef.current = window.setTimeout(() => {
        // Restore overflow before fine-tuning so scroll behaviour returns to normal
        try {
          if (overflowPrevRef.current !== null) {
            document.documentElement.style.overflow = overflowPrevRef.current;
          } else {
            document.documentElement.style.overflow = '';
          }
        } catch (e) {
          // ignore
        }

        resizeWindow();
        followUpTimeoutRef.current = null;
      }, 180);
    };
    document.addEventListener('anafis_content_change', onContentChange);

    return () => {
      window.clearTimeout(timer as number);
      if (resizeTimeoutRef.current) {
        window.clearTimeout(resizeTimeoutRef.current as number);
      }
      // restore overflow if we hid it
      try {
        if (overflowPrevRef.current !== null) {
          document.documentElement.style.overflow = overflowPrevRef.current;
        }
      } catch (e) {
        // ignore
      }
      // restore manual resizing if we disabled it earlier
      (async () => {
        try {
          const w = getCurrentWindow();
          if (w && restored && typeof w.setResizable === 'function') {
            try {
              await w.setResizable(true);
            } catch (err) {
              console.warn('Could not restore resizable(true):', err);
            }
          }
        } catch (err) {
          // ignore
        }
      })();
      resizeObserver.disconnect();
      document.removeEventListener('anafis_content_change', onContentChange);
    };
  }, []);

  const handleClose = async (event: React.MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();

    console.log('Close button clicked - using Tauri command');

    try {
      await invoke('close_uncertainty_calculator_window');
      console.log('Close command sent successfully');
    } catch (error) {
      console.error('Failed to close window via command:', error);

      // Fallback to browser close
      try {
        window.close();
        console.log('Browser close attempted as fallback');
      } catch (fallbackError) {
        console.error('Fallback close also failed:', fallbackError);
      }
    }
  };

  const handleDialogClose = () => {
    // This is called when the dialog wants to close (though we removed the close button)
    console.log('Dialog close requested');
  };

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Box
        ref={contentRef}
        sx={{
          // Fixed width to ensure topbar spans fully; only height will change dynamically
          width: '504px',
          height: 'auto',
          minHeight: '450px',
          boxSizing: 'border-box',
          minWidth: '504px',
          backgroundColor: '#0a0a0a',
          display: 'flex',
          flexDirection: 'column',
          border: 'none',
          outline: 'none',
        }}
      >
        {/* Custom Title Bar */}
        <Box
          // vendor property cast to any to satisfy React.CSSProperties typing
          style={{ WebkitAppRegion: 'drag' } as any}
          sx={{
            width: '100%',
            minWidth: '400px',
            height: '32px',
            background: 'linear-gradient(135deg, #2a2a2a 0%, #3a3a3a 100%)',
            borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            px: 2,
            userSelect: 'none',
            WebkitUserSelect: 'none',
            // Ensure the raw CSS property is present for Tauri/Chromium
            '-webkit-app-region': 'drag',
            position: 'sticky',
            top: 0,
            zIndex: 1200,
            boxShadow: '0 1px 3px rgba(0, 0, 0, 0.3)',
            flexShrink: 0,
          }}
        >
          {/* Window Title */}
          {/* Left spacer to balance the right controls and keep title centered */}
          <Box sx={{ width: '40px', WebkitAppRegion: 'no-drag' }} />

          <Box style={{ WebkitAppRegion: 'drag' } as any} sx={{ display: 'flex', alignItems: 'center', flex: 1, minWidth: 0, justifyContent: 'center' }}>
            <Typography
              variant="body2"
              sx={{
                color: '#ffffff',
                fontWeight: 600,
                fontSize: '0.875rem',
                whiteSpace: 'nowrap',
                overflow: 'hidden',
                textOverflow: 'ellipsis',
                opacity: 0.95,
                pointerEvents: 'none', // avoid capturing clicks so drag works reliably
                textAlign: 'center',
              }}
            >
              Uncertainty Calculator
            </Typography>
          </Box>

          {/* Close Button */}
          <Box
            style={{ WebkitAppRegion: 'no-drag' } as any}
            sx={{
              display: 'flex',
              alignItems: 'center',
              width: '32px',
              justifyContent: 'center',
            }}
          >
            <IconButton
              onClick={handleClose}
              sx={{
                // ensure the button itself does not claim the drag region
                // vendor property omitted from sx due to typing; using wrapper Box style above
                width: '32px',
                height: '32px',
                borderRadius: 0,
                color: 'rgba(255, 255, 255, 0.8)',
                transition: 'all 0.2s ease-in-out',
                '&:hover': {
                  backgroundColor: '#ff4444',
                  color: '#ffffff',
                },
                '&:active': {
                  backgroundColor: '#cc0000',
                },
              }}
            >
              <Close sx={{ fontSize: '16px' }} />
            </IconButton>
          </Box>
        </Box>

        {/* Main Content */}
    <Box
          sx={{
            flex: 1,
            display: 'flex',
            alignItems: 'flex-start',
            justifyContent: 'flex-start',
            p: 1,
      // keep overflow hidden here to avoid internal scrollbars while we control native window size
      overflow: 'hidden',
            minHeight: 0,
          }}
        >
          <UncertaintyCalculatorDialog
            isOpen={true}
            onClose={handleDialogClose}
          />
        </Box>
      </Box>
    </ThemeProvider>
  );
}

export default UncertaintyCalculatorWindow;

// Auto-render immediately when this module loads
console.log('UncertaintyCalculatorWindow module loaded');

const renderUncertaintyCalculatorWindow = () => {
  console.log('UncertaintyCalculatorWindow: Attempting to render');
  const container = document.getElementById('root');
  if (container) {
    console.log('UncertaintyCalculatorWindow: Found root container, creating root');
    try {
      const root = createRoot(container);
      root.render(<UncertaintyCalculatorWindow />);
      console.log('UncertaintyCalculatorWindow: Successfully rendered');
    } catch (error) {
      console.error('UncertaintyCalculatorWindow: Error rendering:', error);
    }
  } else {
    console.error('UncertaintyCalculatorWindow: Root container not found');
  }
};

// Try to render immediately
if (document.readyState === 'complete') {
  renderUncertaintyCalculatorWindow();
} else {
  // Wait for DOM to be ready
  window.addEventListener('load', renderUncertaintyCalculatorWindow);
}
