import { createRoot } from 'react-dom/client';
import './index.css';
import { ThemeProvider } from '@mui/material';
import CssBaseline from '@mui/material/CssBaseline';
import App from '@/App.tsx';
import { createAnafisTheme } from '@/tabs/spreadsheet/components/sidebar/themes/index.ts';

// Prevent pinch-to-zoom on trackpad/touchpad
document.addEventListener(
  'wheel',
  (e) => {
    if (e.ctrlKey) {
      e.preventDefault();
    }
  },
  { passive: false }
);

// Prevent gesture-based zoom on touchscreens
document.addEventListener('gesturestart', (e) => {
  e.preventDefault();
});

document.addEventListener('gesturechange', (e) => {
  e.preventDefault();
});

document.addEventListener('gestureend', (e) => {
  e.preventDefault();
});

// Create theme using shared configuration
const theme = createAnafisTheme();

const rootElement = document.getElementById('root');
if (rootElement) {
  createRoot(rootElement).render(
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <App />
    </ThemeProvider>
  );
}
