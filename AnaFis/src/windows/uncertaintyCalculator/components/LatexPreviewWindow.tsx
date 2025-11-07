import React, { useEffect, useRef, useCallback, memo } from 'react';
import { BlockMath } from 'react-katex';
import 'katex/dist/katex.min.css';
import { invoke } from '@tauri-apps/api/core';
import { createRoot } from 'react-dom/client';
import { ThemeProvider, Box, Paper } from '@mui/material';
import CustomTitleBar from '@/shared/components/CustomTitleBar';
import { createAnafisTheme } from '@/tabs/spreadsheet/components/sidebar/themes';

interface LatexPreviewWindowProps {
  formula: string;
  title: string;
}

// Memoized formula renderer component
const FormulaRenderer = memo(({ formula }: { formula: string }) => {
  if (!formula) {
    return (
      <Box sx={{ color: 'text.secondary', fontStyle: 'italic', textAlign: 'center' }}>
        No formula provided
      </Box>
    );
  }

  return (
    <Box sx={{ fontSize: '1.2em', color: 'text.primary' }}>
      <BlockMath math={formula} />
    </Box>
  );
});

FormulaRenderer.displayName = 'FormulaRenderer';

const LatexPreviewWindow: React.FC<LatexPreviewWindowProps> = ({ formula, title }) => {
  // Constants for window sizing
  const MAX_CONTENT_WIDTH = 8000; // Maximum width for content and Paper component
  const MIN_CONTENT_WIDTH = 500; // Minimum width for content
  
  // Timeout constants for window resizing operations
  const WINDOW_RESIZE_SETTLE_DELAY = 200; // Delay to allow DOM to settle before measuring container size
  const INITIAL_RESIZE_DELAY = 300; // Initial delay after setting document title before first resize
  const RESIZE_DEBOUNCE_DELAY = 100; // Debounce delay for resize observer to prevent excessive resizing

  const containerRef = useRef<HTMLDivElement>(null);
  const resizeObserverRef = useRef<ResizeObserver | null>(null);
  const resizeTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  const adjustWindowSize = useCallback(async () => {
    if (!containerRef.current) {return;}

    await new Promise(resolve => setTimeout(resolve, WINDOW_RESIZE_SETTLE_DELAY));

    const container = containerRef.current;
    const rect = container.getBoundingClientRect();
    const contentWidth = Math.max(MIN_CONTENT_WIDTH, Math.min(MAX_CONTENT_WIDTH, rect.width + 120));
    const fixedHeight = 150 + 32; // 150px content + 32px title bar

    try {
      await invoke('set_window_size', {
        window_id: 'latex-preview',
        width: Math.round(contentWidth),
        height: fixedHeight
      });
    } catch (e) {
      console.log('Could not resize window:', e);
    }
  }, []);

  useEffect(() => {
    document.title = title;
    
    const timeoutId = setTimeout(() => void adjustWindowSize(), INITIAL_RESIZE_DELAY);

    // Setup resize observer
    if (containerRef.current) {
      resizeObserverRef.current = new ResizeObserver(() => {
        if (resizeTimeoutRef.current) {
          clearTimeout(resizeTimeoutRef.current);
        }
        resizeTimeoutRef.current = setTimeout(() => void adjustWindowSize(), RESIZE_DEBOUNCE_DELAY);
      });
      resizeObserverRef.current.observe(containerRef.current);
    }

    return () => {
      clearTimeout(timeoutId);
      if (resizeTimeoutRef.current) {
        clearTimeout(resizeTimeoutRef.current);
      }
      if (resizeObserverRef.current) {
        resizeObserverRef.current.disconnect();
        resizeObserverRef.current = null;
      }
    };
  }, [title, adjustWindowSize]);

  return (
    <Box sx={{
      width: '100vw',
      height: '100vh',
      display: 'flex',
      flexDirection: 'column',
      bgcolor: 'background.default',
      overflow: 'hidden'
    }}>
      <CustomTitleBar title="LaTeX Formula Preview" />

      <Box
        ref={containerRef}
        sx={{
          flex: 1,
          p: 2.5,
          overflow: 'auto',
          '&::-webkit-scrollbar': {
            width: '16px',
            height: '16px',
          },
          '&::-webkit-scrollbar-track': {
            bgcolor: 'background.default',
            borderRadius: 1,
          },
          '&::-webkit-scrollbar-thumb': {
            bgcolor: '#5a5a5a',
            borderRadius: 1,
            border: '3px solid',
            borderColor: 'background.default',
            minHeight: '20px',
            minWidth: '20px',
          },
          '&::-webkit-scrollbar-thumb:hover': {
            bgcolor: '#7a7a7a',
          },
          '&::-webkit-scrollbar-corner': {
            bgcolor: 'background.default',
          },
        }}
      >
        <Paper
          elevation={2}
          sx={{
            p: 2.5,
            width: 'fit-content',
            minWidth: '100%',
            maxWidth: `${MAX_CONTENT_WIDTH}px`,
            display: 'inline-block',
            textAlign: 'center',
          }}
        >
          <FormulaRenderer formula={formula} />
        </Paper>
      </Box>
    </Box>
  );
};

// Auto-render when the module loads
const urlParams = new URLSearchParams(window.location.search);
const formula = decodeURIComponent(urlParams.get('formula') ?? '');
const title = decodeURIComponent(urlParams.get('title') ?? 'LaTeX Preview');

const container = document.getElementById('root');
if (container) {
  // Create theme using shared configuration
  const theme = createAnafisTheme();
  
  // Create and render the component using modern React 18+ API
  const app = React.createElement(ThemeProvider, { theme }, 
    React.createElement(LatexPreviewWindow, { formula, title })
  );
  const root = createRoot(container);
  root.render(app);
}

export default LatexPreviewWindow;
