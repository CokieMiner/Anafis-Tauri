import { createRoot } from 'react-dom/client';
import { useEffect, useRef } from 'react';
import { ThemeProvider } from '@mui/material';
import CssBaseline from '@mui/material/CssBaseline';
import Box from '@mui/material/Box';
import { Typography, IconButton } from '@mui/material';
import { Close } from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';
import UncertaintyCalculatorDialog from './dialogs/UncertaintyCalculatorDialog';
import { createAnafisTheme } from './themes';

// Create theme using shared configuration
const theme = createAnafisTheme();

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
  const height = Math.max(Math.ceil(measuredHeight - 10), 450);

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
        } catch {
          // Failed to resize window
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
    const height = Math.max(Math.ceil(measuredHeight + 10 + extraHeight), 450);

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
    } catch {
      // Failed to resize window (immediate)
    }
  };

  useEffect(() => {
    // Initial resize after component mounts
    const timer = window.setTimeout(resizeWindow, 500);

    // Immediately hide overflow and perform an immediate oversize resize to avoid transient scrollbars
    try {
      overflowPrevRef.current = document.documentElement.style.overflow || null;
      document.documentElement.style.overflow = 'hidden';
    } catch {
      // ignore
    }

    // Do an immediate larger resize to reduce flicker while the app finishes mounting
    (async () => {
      try {
        await resizeNow(60);
      } catch {
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
        } catch {
          // ignore
        }

        resizeWindow();
        followUpTimeoutRef.current = null;
      }, 300);
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
      } catch {
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
        } catch {
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
      } catch {
        // ignore
      }
      resizeObserver.disconnect();
      document.removeEventListener('anafis_content_change', onContentChange);
    };
  }, []);

  const handleClose = async (event: React.MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();

    try {
      await invoke('close_uncertainty_calculator_window');
    } catch {
      // Failed to close window via command

      // Fallback to browser close
      try {
        window.close();
      } catch {
        // Fallback close also failed
      }
    }
  };

  const handleDialogClose = () => {
    // This is called when the dialog wants to close (though we removed the close button)
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
          // vendor property cast to React.CSSProperties with WebkitAppRegion
          style={{ WebkitAppRegion: 'drag' } as React.CSSProperties & { WebkitAppRegion: string }}
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

          <Box style={{ WebkitAppRegion: 'drag' } as React.CSSProperties & { WebkitAppRegion: string }} sx={{ display: 'flex', alignItems: 'center', flex: 1, minWidth: 0, justifyContent: 'center' }}>
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
            style={{ WebkitAppRegion: 'no-drag' } as React.CSSProperties & { WebkitAppRegion: string }}
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
const renderUncertaintyCalculatorWindow = () => {
  const container = document.getElementById('root');
  if (container) {
    try {
      const root = createRoot(container);
      root.render(<UncertaintyCalculatorWindow />);
    } catch {
      // UncertaintyCalculatorWindow: Error rendering
    }
  } else {
    // UncertaintyCalculatorWindow: Root container not found
  }
};

// Try to render immediately
if (document.readyState === 'complete') {
  renderUncertaintyCalculatorWindow();
} else {
  // Wait for DOM to be ready
  window.addEventListener('load', renderUncertaintyCalculatorWindow);
}
