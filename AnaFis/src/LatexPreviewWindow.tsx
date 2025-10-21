import React, { useEffect, useRef } from 'react';
import { BlockMath } from 'react-katex';
import 'katex/dist/katex.min.css';
import { invoke } from '@tauri-apps/api/core';
import { createRoot } from 'react-dom/client';
import CustomTitleBar from './components/CustomTitleBar';
import { ThemeProvider, useTheme } from '@mui/material';
import { createAnafisTheme } from './themes';

interface LatexPreviewWindowProps {
  formula: string;
  title: string;
}

const LatexPreviewWindow: React.FC<LatexPreviewWindowProps> = ({ formula, title }) => {
  const theme = useTheme();
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    document.title = title;

    const adjustWindowSize = async () => {
      if (containerRef.current) {
        await new Promise(resolve => setTimeout(resolve, 200));

        const container = containerRef.current;
        container.style.display = 'none';
        void container.offsetHeight; // Force reflow
        container.style.display = 'block';

        const rect = container.getBoundingClientRect();
        const contentWidth = Math.max(500, Math.min(10000, rect.width + 120)); // Increased max width for very long formulas
        const fixedHeight = 150 + 32; // 150px content + 32px title bar

        try {
          await invoke('set_window_size', {
            width: Math.round(contentWidth),
            height: fixedHeight
          });
        } catch (e) {
          console.log('Could not resize window:', e);
        }
      }
    };

    setTimeout(adjustWindowSize, 300);

    const resizeObserver = new ResizeObserver(() => {
      setTimeout(adjustWindowSize, 100);
    });

    if (containerRef.current) {
      resizeObserver.observe(containerRef.current);
    }

    return () => resizeObserver.disconnect();
  }, [formula, title]);

  return (
    <div style={{
      margin: 0,
      padding: 0,
      fontFamily: 'system-ui, Avenir, Helvetica, Arial, sans-serif',
      backgroundColor: '#0a0a0a',
      color: '#ffffff',
      width: '100vw',
      height: '100vh',
      overflow: 'hidden'
    }}>
      <CustomTitleBar title="LaTeX Formula Preview" />

      {/* Content */}
      <div
        ref={containerRef}
        style={{
          padding: '20px',
          height: '135px', // Fixed height for content area
          overflowX: 'auto',
          overflowY: 'auto',
          backgroundColor: theme.palette.background.default,
          scrollbarWidth: 'thin',
          scrollbarColor: '#555 #0a0a0a'
        }}
        data-scrollbar-container
      >
        <div style={{
          backgroundColor: theme.palette.background.paper,
          borderRadius: '8px',
          padding: '20px',
          border: '1px solid rgba(255, 255, 255, 0.1)',
          width: 'fit-content',
          minWidth: '100%',
          maxWidth: '8000px', // Increased for very long formulas
          boxShadow: '0 2px 8px rgba(0, 0, 0, 0.3)',
          display: 'inline-block',
          overflow: 'visible'
        }}>
          {formula ? (
            <div style={{
              fontSize: '1.2em',
              color: theme.palette.text.primary,
              display: 'block',
              overflow: 'visible',
              whiteSpace: 'normal'
            }}>
              <BlockMath math={formula} />
            </div>
          ) : (
            <div style={{ color: theme.palette.text.secondary, fontStyle: 'italic' }}>
              No formula provided
            </div>
          )}
        </div>
      </div>

      {/* Scrollbar Styles */}
      <style dangerouslySetInnerHTML={{
        __html: `
          /* Custom scrollbar for the content area */
          div[data-scrollbar-container]::-webkit-scrollbar {
            width: 16px !important;
            height: 16px !important;
            background: ${theme.palette.background.default} !important;
          }

          div[data-scrollbar-container]::-webkit-scrollbar-track {
            background: ${theme.palette.background.default} !important;
            border-radius: 8px !important;
          }

          div[data-scrollbar-container]::-webkit-scrollbar-thumb {
            background: #5a5a5a !important;
            border-radius: 8px !important;
            border: 3px solid ${theme.palette.background.default} !important;
            min-height: 20px !important;
            min-width: 20px !important;
          }

          div[data-scrollbar-container]::-webkit-scrollbar-thumb:hover {
            background: #7a7a7a !important;
          }

          div[data-scrollbar-container]::-webkit-scrollbar-corner {
            background: ${theme.palette.background.default} !important;
          }

          /* Ensure scrollbars are always visible */
          div[data-scrollbar-container] {
            scrollbar-width: thin !important;
            scrollbar-color: #5a5a5a ${theme.palette.background.default} !important;
          }
        `
      }} />
    </div>
  );
};

// Auto-render when the module loads
const urlParams = new URLSearchParams(window.location.search);
const formula = decodeURIComponent(urlParams.get('formula') || '');
const title = decodeURIComponent(urlParams.get('title') || 'LaTeX Preview');

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
