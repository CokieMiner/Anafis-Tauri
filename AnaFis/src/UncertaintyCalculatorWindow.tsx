import { createRoot } from 'react-dom/client';
import { useEffect, useRef } from 'react';
import { ThemeProvider } from '@mui/material';
import CssBaseline from '@mui/material/CssBaseline';
import Box from '@mui/material/Box';
import { invoke } from '@tauri-apps/api/core';
import UncertaintyCalculatorDialog from './dialogs/UncertaintyCalculatorDialog';
import { createAnafisTheme } from './themes';
import CustomTitleBar from './components/CustomTitleBar';

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
        <CustomTitleBar title="Uncertainty Calculator" />

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
          <UncertaintyCalculatorDialog />
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
