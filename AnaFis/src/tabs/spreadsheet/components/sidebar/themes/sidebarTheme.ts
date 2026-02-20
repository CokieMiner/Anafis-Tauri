// AnaFis Sidebar Theme - Unified design system for all sidebars
// Inspired by Home tab aesthetics with blue accents

// Color constants for better performance and consistency
const COLORS = {
  PRIMARY: '#2196f3',
  PRIMARY_DARK: '#1976d2',
  SECONDARY: '#64b5f6',
  ACCENT: '#90caf9',
  SUCCESS: '#4caf50',
  WARNING: '#ff9800',
  ERROR: '#f44336',
} as const;

const TEXT_COLORS = {
  PRIMARY: 'rgba(255, 255, 255, 0.95)',
  SECONDARY: 'rgba(255, 255, 255, 0.8)',
  TERTIARY: 'rgba(255, 255, 255, 0.6)',
  DISABLED: 'rgba(255, 255, 255, 0.4)',
} as const;

// Shared hover effects to reduce duplication
const HOVER_EFFECTS = {
  primary: {
    transform: 'translateY(-1px)',
    boxShadow: '0 4px 15px rgba(33, 150, 243, 0.4)',
  },
} as const;

export const sidebarTheme = {
  // Color Palette
  colors: {
    primary: COLORS.PRIMARY,
    primaryDark: COLORS.PRIMARY_DARK,
    primaryContrast: TEXT_COLORS.PRIMARY,
    secondary: COLORS.SECONDARY,
    accent: COLORS.ACCENT,
    success: COLORS.SUCCESS,
    warning: COLORS.WARNING,
    error: COLORS.ERROR,
    hover: COLORS.PRIMARY_DARK,
    disabledBg: '#555555',
    disabledText: 'rgba(255, 255, 255, 0.5)',
    text: TEXT_COLORS,
  },

  // Background Gradients (Home tab inspired)
  backgrounds: {
    container:
      'radial-gradient(circle at 20% 50%, rgba(30, 27, 75, 0.05) 0%, transparent 50%), radial-gradient(circle at 80% 20%, rgba(127, 29, 29, 0.05) 0%, transparent 50%), radial-gradient(circle at 40% 80%, rgba(88, 28, 135, 0.05) 0%, transparent 50%)',
    header:
      'radial-gradient(circle at 20% 50%, rgba(30, 27, 75, 0.05) 0%, transparent 50%), radial-gradient(circle at 80% 20%, rgba(127, 29, 29, 0.05) 0%, transparent 50%), radial-gradient(circle at 40% 80%, rgba(88, 28, 135, 0.05) 0%, transparent 50%)',
    card: 'linear-gradient(135deg, rgba(26, 26, 26, 0.8) 0%, rgba(42, 42, 42, 0.4) 100%)',
    section: 'rgba(33, 150, 243, 0.05)',
    hover: 'rgba(33, 150, 243, 0.1)',
  },

  // Border Styles
  borders: {
    container: '1px solid rgba(33, 150, 243, 0.3)',
    leftAccent: '3px solid #2196f3',
    section: '1px solid rgba(33, 150, 243, 0.2)',
    card: '1px solid rgba(33, 150, 243, 0.1)',
    focus: '1px solid #2196f3',
  },

  // Shadow Effects
  shadows: {
    container: '0 8px 32px rgba(33, 150, 243, 0.2)',
    card: '0 4px 15px rgba(33, 150, 243, 0.1)',
    cardHover: '0 8px 25px rgba(33, 150, 243, 0.15)',
    button: '0 4px 15px rgba(33, 150, 243, 0.4)',
  },

  // Typography Scale
  typography: {
    header: {
      fontSize: '1.5rem',
      fontWeight: 700,
      color: '#2196f3',
      lineHeight: 1.2,
    },
    sectionTitle: {
      fontSize: '1rem',
      fontWeight: 600,
      color: '#64b5f6',
      textTransform: 'uppercase',
      letterSpacing: '0.5px',
      lineHeight: 1.3,
    },
    label: {
      fontSize: '0.875rem',
      color: 'rgba(255, 255, 255, 0.9)',
      fontWeight: 500,
      lineHeight: 1.4,
    },
    caption: {
      fontSize: '0.75rem',
      color: 'rgba(255, 255, 255, 0.7)',
      lineHeight: 1.5,
    },
    body: {
      fontSize: '0.875rem',
      color: 'rgba(255, 255, 255, 0.9)',
      lineHeight: 1.6,
    },
  },

  // Spacing Scale
  spacing: {
    container: 2.5, // 20px
    section: 2, // 16px
    element: 1.5, // 12px
    tight: 1, // 8px
    loose: 3, // 24px
  },

  // Border Radius
  radius: {
    container: '12px 0 0 12px',
    card: '12px',
    button: '8px',
    input: '6px',
  },

  // Transitions
  transitions: {
    default: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
    hover: 'all 0.2s ease-in-out',
    focus: 'all 0.15s ease-out',
    transform: 'transform 0.2s ease-in-out',
  },

  // Component-specific styles
  components: {
    button: {
      primary: {
        background: 'linear-gradient(135deg, #2196f3 0%, #1976d2 100%)',
        color: 'white',
        '&:hover': {
          background: 'linear-gradient(135deg, #1976d2 0%, #1565c0 100%)',
          ...HOVER_EFFECTS.primary,
        },
      },
      secondary: {
        borderColor: 'rgba(33, 150, 243, 0.5)',
        color: '#64b5f6',
        '&:hover': {
          borderColor: '#2196f3',
          backgroundColor: 'rgba(33, 150, 243, 0.1)',
          transform: 'translateY(-1px)',
        },
      },
      // Default button style for all sidebar buttons
      default: {
        backgroundColor: COLORS.PRIMARY,
        color: 'white',
        border: 'none',
        '&:hover': {
          backgroundColor: COLORS.PRIMARY_DARK,
          ...HOVER_EFFECTS.primary,
        },
      },
    },
    input: {
      root: {
        '& .MuiOutlinedInput-root': {
          backgroundColor: 'rgba(33, 150, 243, 0.05)',
          borderRadius: '6px',
          '& fieldset': {
            borderColor: 'rgba(33, 150, 243, 0.2)',
          },
          '&:hover fieldset': {
            borderColor: 'rgba(33, 150, 243, 0.4)',
          },
          '&.Mui-focused fieldset': {
            borderColor: '#2196f3',
          },
        },
        '& .MuiInputLabel-root': {
          color: '#64b5f6',
          '&.Mui-focused': {
            color: '#2196f3',
          },
        },
        '& input': {
          color: 'rgba(255, 255, 255, 0.9)',
        },
      },
    },
  },
} as const;

// Type exports for TypeScript
export type SidebarTheme = typeof sidebarTheme;
export type SidebarColors = typeof sidebarTheme.colors;
export type SidebarTypography = typeof sidebarTheme.typography;
