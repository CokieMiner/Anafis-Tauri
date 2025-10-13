import { createTheme } from '@mui/material/styles';

// Shared theme configuration for AnaFis
export const createAnafisTheme = () => {
  return createTheme({
    palette: {
      mode: 'dark',
      primary: {
        main: '#9c27b0', // Readable purple for settings
        light: '#ba68c8',
        dark: '#7b1fa2',
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
            backgroundImage: 'linear-gradient(135deg, rgba(33, 150, 243, 0.15) 0%, rgba(33, 150, 243, 0.05) 100%)',
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
            },
          },
        },
      },
      MuiTextField: {
        styleOverrides: {
          root: {
            '& .MuiOutlinedInput-root': {
              '&.Mui-focused fieldset': {
                borderColor: '#2196f3',
              },
            },
            '& .MuiInputLabel-root.Mui-focused': {
              color: '#2196f3',
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
};