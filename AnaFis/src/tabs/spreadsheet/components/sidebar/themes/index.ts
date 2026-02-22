import { createTheme } from '@mui/material/styles';
import { anafisTheme } from '@/shared/theme/unifiedTheme';

// Re-export anafisColors for backwards compatibility
// This now derives from the unified theme
export const anafisColors = {
  // Primary theme colors
  primary: anafisTheme.colors.primary.main,
  spreadsheet: anafisTheme.colors.tabs.spreadsheet.main,

  // Tab-specific colors
  tabs: {
    home: anafisTheme.colors.tabs.home.main,
    spreadsheet: anafisTheme.colors.tabs.spreadsheet.main,
    fitting: anafisTheme.colors.tabs.fitting.main,
    solver: anafisTheme.colors.tabs.solver.main,
    montecarlo: anafisTheme.colors.tabs.montecarlo.main,
  },

  // Top bar button colors (bright versions)
  buttons: {
    minimize: anafisTheme.colors.windowControls.minimize.main,
    maximize: anafisTheme.colors.windowControls.maximize.main,
    close: anafisTheme.colors.windowControls.close.main,
  },

  // UI element colors
  ui: {
    background: anafisTheme.colors.background.primary,
    paper: anafisTheme.colors.background.paper,
    border: anafisTheme.colors.border.default,
    text: {
      primary: anafisTheme.colors.text.primary,
      secondary: anafisTheme.colors.text.secondary,
      tertiary: anafisTheme.colors.text.tertiary,
    },
  },
};

// Memoized theme instances for performance
let _anafisTheme: ReturnType<typeof createTheme> | null = null;
let _noTransitionTheme: ReturnType<typeof createTheme> | null = null;

// Shared theme configuration for AnaFis
export const createAnafisTheme = () => {
  _anafisTheme ??= createTheme({
    palette: {
      mode: 'dark',
      primary: {
        main: anafisColors.primary, // Purple as default
        light: '#ba68c8',
        dark: '#7b1fa2',
      },
      secondary: {
        main: anafisColors.spreadsheet, // Blue for spreadsheet
        light: '#64b5f6',
        dark: '#1976d2',
      },
      background: {
        default: anafisColors.ui.background,
        paper: anafisColors.ui.paper,
      },
      text: {
        primary: anafisColors.ui.text.primary,
        secondary: anafisColors.ui.text.secondary,
      },
      success: {
        main: anafisColors.buttons.minimize, // Green
        light: '#81c784',
        dark: '#388e3c',
      },
      warning: {
        main: anafisColors.tabs.fitting, // Orange
        light: '#ffb74d',
        dark: '#f57c00',
      },
      error: {
        main: anafisColors.buttons.close, // Red
        light: '#ef5350',
        dark: '#d32f2f',
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
            backgroundImage:
              'linear-gradient(135deg, rgba(33, 150, 243, 0.15) 0%, rgba(33, 150, 243, 0.05) 100%)',
            backdropFilter: 'blur(10px)',
            borderBottom: '1px solid rgba(255, 255, 255, 0.08)',
          },
        },
      },
      MuiPaper: {
        styleOverrides: {
          root: {
            backgroundColor: '#111111',
            backgroundImage:
              'linear-gradient(135deg, rgba(255, 255, 255, 0.03) 0%, rgba(255, 255, 255, 0.01) 100%)',
            border: '1px solid rgba(255, 255, 255, 0.08)',
            transition: 'none',
          },
        },
      },
      MuiButton: {
        styleOverrides: {
          root: {
            borderRadius: 8,
            padding: '8px 16px',
            transition: 'none',
            '&:hover': {
              transform: 'none',
            },
          },
        },
      },
      MuiTextField: {
        styleOverrides: {
          root: {
            '& .MuiOutlinedInput-root': {
              '&.Mui-focused fieldset': {
                borderColor: '#9c27b0', // Use purple primary color
              },
            },
            '& .MuiInputLabel-root.Mui-focused': {
              color: '#9c27b0', // Use purple primary color
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
      MuiBackdrop: {
        styleOverrides: {
          root: {
            backgroundColor: 'rgba(0, 0, 0, 0.8)',
          },
        },
      },
      MuiCircularProgress: {
        styleOverrides: {
          root: {
            color: anafisColors.ui.text.primary,
          },
        },
      },
    },
  });
  return _anafisTheme;
};

// Theme without transitions for windows that need to resize smoothly without flickering
export const createNoTransitionTheme = () => {
  _noTransitionTheme ??= createTheme(createAnafisTheme(), {
    components: {
      MuiFormLabel: {
        styleOverrides: {
          root: {
            transition: 'none',
          },
        },
      },
      MuiRadio: {
        styleOverrides: {
          root: {
            transition: 'none',
          },
        },
      },
      MuiCheckbox: {
        styleOverrides: {
          root: {
            transition: 'none',
          },
        },
      },
      MuiListItemButton: {
        styleOverrides: {
          root: {
            transition: 'none',
          },
        },
      },
    },
  });
  return _noTransitionTheme;
};
