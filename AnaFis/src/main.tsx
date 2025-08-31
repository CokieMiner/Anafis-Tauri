import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'
import { ThemeProvider, createTheme } from '@mui/material';
import CssBaseline from '@mui/material/CssBaseline';

// Create theme immediately
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
    MuiTabs: {
      styleOverrides: {
        root: {
          '& .MuiTabs-indicator': {
            backgroundColor: '#1e1b4b',
            height: 3,
            borderRadius: 3,
          },
        },
      },
    },
    MuiTab: {
      styleOverrides: {
        root: {
          '&.Mui-selected': {
            color: '#1e1b4b',
            fontWeight: 600,
          },
        },
      },
    },
  },
});

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <App />
    </ThemeProvider>
  </StrictMode>,
)
