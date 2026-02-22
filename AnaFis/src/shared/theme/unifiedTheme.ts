/**
 * AnaFis Unified Theme System
 * Single source of truth for all colors, gradients, and design tokens
 *
 * @example
 * import { anafisTheme } from '@/shared/theme/unifiedTheme';
 * const color = anafisTheme.colors.tabs.spreadsheet;
 */

// =============================================================================
// COLOR PALETTE
// =============================================================================

/**
 * Core color palette - all colors derive from these values
 */
const palette = {
  // Primary brand colors
  purple: {
    main: '#9c27b0',
    light: '#ba68c8',
    dark: '#7b1fa2',
    contrast: '#ffffff',
  },

  // Tab-specific accent colors
  blue: {
    main: '#2196f3',
    light: '#64b5f6',
    dark: '#1976d2',
    contrast: '#ffffff',
  },

  amber: {
    main: '#ffb300',
    light: '#ffb74d',
    dark: '#f57c00',
    contrast: '#111111',
  },

  green: {
    main: '#4caf50',
    light: '#81c784',
    dark: '#388e3c',
    contrast: '#ffffff',
  },

  pink: {
    main: '#e91e63',
    light: '#f06292',
    dark: '#c2185b',
    contrast: '#ffffff',
  },

  red: {
    main: '#f44336',
    light: '#ef5350',
    dark: '#d32f2f',
    contrast: '#ffffff',
  },

  // Neutral colors
  neutral: {
    white: '#ffffff',
    black: '#000000',
    gray100: '#f5f5f5',
    gray200: '#e0e0e0',
    gray500: '#9e9e9e',
    gray700: '#616161',
    gray900: '#212121',
  },
} as const;

// =============================================================================
// SEMANTIC COLORS
// =============================================================================

/**
 * Semantic color assignments for the application
 */
export const anafisTheme = {
  // ---------------------------------------------------------------------------
  // COLORS
  // ---------------------------------------------------------------------------
  colors: {
    // Primary theme color (purple)
    primary: palette.purple,

    // Secondary theme color (blue)
    secondary: palette.blue,

    // Tab-specific colors
    tabs: {
      home: palette.purple,
      spreadsheet: palette.blue,
      fitting: palette.amber,
      solver: palette.green,
      montecarlo: palette.pink,
    },

    // Window control button colors
    windowControls: {
      minimize: palette.green,
      maximize: palette.blue,
      close: palette.red,
    },

    // Status colors
    status: {
      success: palette.green,
      warning: palette.amber,
      error: palette.red,
      info: palette.blue,
    },

    // Background colors
    background: {
      primary: '#0a0a0a',
      secondary: '#1a1a1a',
      paper: '#222222',
      elevated: '#2a2a2a',
    },

    // Text colors
    text: {
      primary: 'rgba(255, 255, 255, 0.95)',
      secondary: 'rgba(255, 255, 255, 0.8)',
      tertiary: 'rgba(255, 255, 255, 0.6)',
      disabled: 'rgba(255, 255, 255, 0.4)',
    },

    // Border colors
    border: {
      default: 'rgba(255, 255, 255, 0.08)',
      light: 'rgba(255, 255, 255, 0.12)',
      focus: 'rgba(255, 255, 255, 0.3)',
    },

    // Sidebar specific colors
    sidebar: {
      primary: palette.blue.main,
      primaryDark: palette.blue.dark,
      primaryContrast: palette.neutral.white,
      secondary: palette.blue.light,
      accent: '#90caf9',
      hover: palette.blue.dark,
      disabledBg: '#555555',
      disabledText: 'rgba(255, 255, 255, 0.5)',
    },
  },

  // ---------------------------------------------------------------------------
  // GRADIENTS
  // ---------------------------------------------------------------------------
  gradients: {
    // Title bar gradient
    titleBar: 'linear-gradient(135deg, #1a1a1a 0%, #2a2a2a 100%)',

    // Toolbar gradient
    toolbar:
      'linear-gradient(135deg, rgba(26, 26, 26, 0.95) 0%, rgba(42, 42, 42, 0.95) 100%)',

    // Card backgrounds
    card: 'linear-gradient(135deg, rgba(26, 26, 26, 0.8) 0%, rgba(42, 42, 42, 0.4) 100%)',

    // Primary button gradient
    buttonPrimary: 'linear-gradient(135deg, #2196f3 0%, #1976d2 100%)',
    buttonPrimaryHover: 'linear-gradient(135deg, #1976d2 0%, #1565c0 100%)',

    // Background radial gradients (consistent across all views)
    backgroundRadial: [
      'radial-gradient(circle at 20% 50%, rgba(30, 27, 75, 0.05) 0%, transparent 50%)',
      'radial-gradient(circle at 80% 20%, rgba(127, 29, 29, 0.05) 0%, transparent 50%)',
      'radial-gradient(circle at 40% 80%, rgba(88, 28, 135, 0.05) 0%, transparent 50%)',
    ].join(', '),

    // Fitting tab specific (slightly more visible)
    backgroundRadialFitting: [
      'radial-gradient(circle at 20% 50%, rgba(30, 27, 75, 0.12) 0%, transparent 52%)',
      'radial-gradient(circle at 80% 20%, rgba(127, 29, 29, 0.10) 0%, transparent 48%)',
      'radial-gradient(circle at 45% 82%, rgba(88, 28, 135, 0.09) 0%, transparent 44%)',
    ].join(', '),

    // Sidebar background
    sidebar:
      'linear-gradient(135deg, rgba(26, 26, 26, 0.8) 0%, rgba(42, 42, 42, 0.4) 100%)',
  },

  // ---------------------------------------------------------------------------
  // SHADOWS
  // ---------------------------------------------------------------------------
  shadows: {
    none: 'none',
    sm: '0 1px 3px rgba(0, 0, 0, 0.3)',
    md: '0 4px 15px rgba(0, 0, 0, 0.3)',
    lg: '0 8px 32px rgba(0, 0, 0, 0.4)',
    xl: '0 20px 40px rgba(0, 0, 0, 0.5)',
    // Colored shadows
    primary: '0 4px 15px rgba(33, 150, 243, 0.4)',
    primaryHover: '0 8px 25px rgba(33, 150, 243, 0.15)',
    card: '0 4px 15px rgba(33, 150, 243, 0.1)',
    cardHover: '0 8px 25px rgba(33, 150, 243, 0.15)',
    container: '0 8px 32px rgba(33, 150, 243, 0.2)',
  },

  // ---------------------------------------------------------------------------
  // SPACING
  // ---------------------------------------------------------------------------
  spacing: {
    xs: 0.5, // 4px
    sm: 1, // 8px
    md: 1.5, // 12px
    lg: 2, // 16px
    xl: 2.5, // 20px
    xxl: 3, // 24px
  },

  // ---------------------------------------------------------------------------
  // BORDER RADIUS
  // ---------------------------------------------------------------------------
  radius: {
    none: '0',
    sm: '4px',
    md: '6px',
    lg: '8px',
    xl: '12px',
    full: '9999px',
    // Component-specific
    button: '8px',
    card: '12px',
    container: '12px 0 0 12px',
    input: '6px',
  },

  // ---------------------------------------------------------------------------
  // TRANSITIONS
  // ---------------------------------------------------------------------------
  transitions: {
    default: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
    hover: 'all 0.2s ease-in-out',
    focus: 'all 0.15s ease-out',
    transform: 'transform 0.2s ease-in-out',
    none: 'none',
  },

  // ---------------------------------------------------------------------------
  // TYPOGRAPHY
  // ---------------------------------------------------------------------------
  typography: {
    fontFamily: '"Inter", "Roboto", "Helvetica", "Arial", sans-serif',
    header: {
      fontSize: '1.5rem',
      fontWeight: 700,
      lineHeight: 1.2,
    },
    sectionTitle: {
      fontSize: '1rem',
      fontWeight: 600,
      textTransform: 'uppercase' as const,
      letterSpacing: '0.5px',
      lineHeight: 1.3,
    },
    label: {
      fontSize: '0.875rem',
      fontWeight: 500,
      lineHeight: 1.4,
    },
    caption: {
      fontSize: '0.75rem',
      lineHeight: 1.5,
    },
    body: {
      fontSize: '0.875rem',
      lineHeight: 1.6,
    },
  },
} as const;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/**
 * Get tab-specific colors by tab type
 */
export function getTabColor(tabType: keyof typeof anafisTheme.colors.tabs) {
  return anafisTheme.colors.tabs[tabType] ?? anafisTheme.colors.primary;
}

/**
 * Get background gradient for a specific context
 */
export function getBackgroundGradient(
  context: 'default' | 'fitting' = 'default'
) {
  return context === 'fitting'
    ? anafisTheme.gradients.backgroundRadialFitting
    : anafisTheme.gradients.backgroundRadial;
}

/**
 * Create a colored shadow
 */
export function createColoredShadow(
  color: string,
  opacity: number = 0.4
): string {
  return `0 4px 15px ${color}${Math.round(opacity * 255)
    .toString(16)
    .padStart(2, '0')}`;
}

// =============================================================================
// TYPE EXPORTS
// =============================================================================

export type AnaFisTheme = typeof anafisTheme;
export type TabType = keyof typeof anafisTheme.colors.tabs;
