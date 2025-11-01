import { SxProps, Theme } from '@mui/material';
import { sidebarTheme } from '../themes/sidebarTheme';

// Pre-computed style objects for better performance
const CONTAINER_STYLE: SxProps<Theme> = {
  width: 450,
  height: '100%',
  display: 'flex',
  flexDirection: 'column',
  background: sidebarTheme.backgrounds.container,
  border: sidebarTheme.borders.container,
  borderLeft: sidebarTheme.borders.leftAccent,
  borderRadius: sidebarTheme.radius.container,
  overflow: 'hidden', // Keep hidden to prevent container scroll, let content scroll
  boxShadow: sidebarTheme.shadows.container,
  backdropFilter: 'blur(20px)'
};

const HEADER_STYLE: SxProps<Theme> = {
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'space-between',
  p: sidebarTheme.spacing.container,
  background: sidebarTheme.backgrounds.header,
  borderBottom: sidebarTheme.borders.section,
  backdropFilter: 'blur(10px)'
};

const CONTENT_STYLE: SxProps<Theme> = {
  flex: 1,
  overflow: 'auto',
  p: sidebarTheme.spacing.container,
  background: sidebarTheme.backgrounds.section
};

// Content wrapper for scrollable content areas
const CONTENT_WRAPPER_STYLE: SxProps<Theme> = {
  flex: 1,
  display: 'flex',
  flexDirection: 'column',
  overflow: 'auto', // Changed from 'hidden' to 'auto' to enable scrolling
  gap: sidebarTheme.spacing.element,
  p: sidebarTheme.spacing.container
};

const FOOTER_STYLE: SxProps<Theme> = {
  p: sidebarTheme.spacing.container,
  borderTop: sidebarTheme.borders.section,
  background: sidebarTheme.backgrounds.header,
  display: 'flex',
  gap: sidebarTheme.spacing.element
};

// Shared styling utilities for consistent sidebar design
export const sidebarStyles = {
  // Container styles
  container: CONTAINER_STYLE,

  // Header styles
  header: HEADER_STYLE,

  // Content area
  content: CONTENT_STYLE,

  // Content wrapper for scrollable areas
  contentWrapper: CONTENT_WRAPPER_STYLE,

  // Footer styles
  footer: FOOTER_STYLE,

  // Tip/info box
  tipBox: {
    p: sidebarTheme.spacing.element,
    background: sidebarTheme.backgrounds.hover,
    border: sidebarTheme.borders.card,
    borderRadius: sidebarTheme.radius.card,
    mb: sidebarTheme.spacing.element
  } as SxProps<Theme>,

  // Form section
  formSection: {
    mb: sidebarTheme.spacing.section
  } as SxProps<Theme>,

  // Button styles - All buttons should be blue by default
  button: (() => {
    // Base button properties shared by primary and default variants
    const baseButton = {
      borderRadius: sidebarTheme.radius.button,
      textTransform: 'none',
      px: 3,
      py: 1.5,
      border: 'none',
      '&:hover': {
        backgroundColor: sidebarTheme.colors.hover,
        transform: 'translateY(-1px)',
        boxShadow: sidebarTheme.shadows.button
      },
      '&:disabled': {
        backgroundColor: sidebarTheme.colors.disabledBg,
        color: sidebarTheme.colors.disabledText
      }
    } as const;

    return {
      primary: {
        ...baseButton,
        fontWeight: 600,
        backgroundColor: sidebarTheme.colors.primary,
        color: sidebarTheme.colors.primaryContrast,
        border: `1px solid ${sidebarTheme.colors.primary}`
      } as SxProps<Theme>,

      secondary: {
        borderRadius: sidebarTheme.radius.button,
        fontWeight: 500,
        textTransform: 'none',
        px: 3,
        py: 1.5,
        backgroundColor: 'transparent',
        borderColor: sidebarTheme.colors.primary,
        color: sidebarTheme.colors.primary,
        border: `1px solid ${sidebarTheme.colors.primary}`,
        '&:hover': {
          borderColor: sidebarTheme.colors.primary,
          backgroundColor: sidebarTheme.backgrounds.hover,
          transform: 'translateY(-1px)'
        }
      } as SxProps<Theme>,

      // Default button style that should be applied to all buttons
      default: {
        ...baseButton,
        fontWeight: 500,
        backgroundColor: sidebarTheme.colors.primary,
        color: sidebarTheme.colors.primaryContrast,
        border: `1px solid ${sidebarTheme.colors.primary}`
      } as SxProps<Theme>
    };
  })(),

  // Input styles
  input: {
    ...sidebarTheme.components.input.root,
    mb: sidebarTheme.spacing.element
  } as SxProps<Theme>,

  // Typography styles
  text: {
    header: sidebarTheme.typography.header,
    sectionTitle: sidebarTheme.typography.sectionTitle,
    label: sidebarTheme.typography.label,
    caption: sidebarTheme.typography.caption,
    body: sidebarTheme.typography.body
  },

  // Layout helpers
  flexColumn: {
    display: 'flex',
    flexDirection: 'column'
  } as SxProps<Theme>,

  flexRow: {
    display: 'flex',
    alignItems: 'center'
  } as SxProps<Theme>,

  spaceBetween: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between'
  } as SxProps<Theme>,

  // Two-column layout for sidebars like UnitConverter
  twoColumn: {
    display: 'flex',
    height: '100%',
    gap: sidebarTheme.spacing.element,
    leftColumn: {
      width: 140,
      borderRight: sidebarTheme.borders.section,
      background: sidebarTheme.backgrounds.section,
      borderRadius: sidebarTheme.radius.card,
      overflow: 'auto'
    } as SxProps<Theme>,
    rightColumn: {
      flex: 1,
      overflow: 'auto'
    } as SxProps<Theme>
  },

  // Icon button styles
  iconButton: {
    color: sidebarTheme.colors.text.SECONDARY,
    borderRadius: sidebarTheme.radius.button,
    transition: sidebarTheme.transitions.hover,
    '&:hover': {
      backgroundColor: sidebarTheme.backgrounds.hover,
      color: sidebarTheme.colors.text.PRIMARY,
      transform: 'scale(1.05)'
    }
  } as SxProps<Theme>
};

// Bounded memoization cache for dynamic styles (prevents memory leaks)
const MAX_CACHE_SIZE = 50;
const styleCache = new Map<string, SxProps<Theme>>();

/**
 * Safely add to cache with size limit and LRU eviction
 * If key exists, delete it first to update its position to most recently used
 */
const setCacheWithLimit = (key: string, value: SxProps<Theme>): void => {
  // If key already exists, delete it to update its position
  if (styleCache.has(key)) {
    styleCache.delete(key);
  }

  // If cache is at limit after potential deletion, remove least recently used (first in Map)
  if (styleCache.size >= MAX_CACHE_SIZE) {
    const firstKey = styleCache.keys().next().value;
    if (firstKey) {
      styleCache.delete(firstKey);
    }
  }

  styleCache.set(key, value);
};

/**
 * Get cached value and update its position to most recently used
 */
const getCachedValue = (key: string): SxProps<Theme> | undefined => {
  if (styleCache.has(key)) {
    const value = styleCache.get(key)!;
    // Remove and re-insert to make it most recently used
    styleCache.delete(key);
    styleCache.set(key, value);
    return value;
  }
  return undefined;
};

// Utility function to convert hex color to rgba with proper validation
const hexToRgba = (hex: string, alpha: number): string => {
  // Validate and normalize hex input
  if (typeof hex !== 'string') {
    throw new Error('hexToRgba: hex parameter must be a string');
  }

  // Pre-validate hex string format
  if (!/^[#]?[0-9a-fA-F]{3}([0-9a-fA-F]{3})?$/.test(hex)) {
    throw new Error(`hexToRgba: invalid hex color format '${hex}'. Expected 3 or 6 character hex code with optional # prefix`);
  }

  // Remove # if present and convert to lowercase
  let cleanHex = hex.replace('#', '').toLowerCase();

  // Handle 3-character hex codes by duplicating each nibble
  if (cleanHex.length === 3) {
    cleanHex = cleanHex.split('').map(char => char + char).join('');
  }

  // Validate hex format: must be exactly 6 characters and contain only valid hex characters
  if (cleanHex.length !== 6 || !/^[0-9a-f]{6}$/.test(cleanHex)) {
    throw new Error(`hexToRgba: invalid hex color format '${hex}'. Expected 3 or 6 character hex code with valid characters [0-9a-fA-F]`);
  }

  // Validate and clamp alpha to 0.0-1.0 range
  if (typeof alpha !== 'number' || isNaN(alpha)) {
    throw new Error('hexToRgba: alpha parameter must be a valid number');
  }

  const clampedAlpha = Math.max(0, Math.min(1, alpha));

  // Warn if alpha was out of range and got clamped
  if (clampedAlpha !== alpha) {
    console.warn(`hexToRgba: alpha value ${alpha} was clamped to ${clampedAlpha} (valid range: 0.0-1.0)`);
  }

  // Parse hex values (guaranteed valid after validation)
  const r = parseInt(cleanHex.substring(0, 2), 16);
  const g = parseInt(cleanHex.substring(2, 4), 16);
  const b = parseInt(cleanHex.substring(4, 6), 16);

  // Return well-formed rgba string
  return `rgba(${r}, ${g}, ${b}, ${clampedAlpha})`;
};
// Utility functions for dynamic styles with bounded memoization
export const getHoverStyles = (color = sidebarTheme.colors.primary): SxProps<Theme> => {
  const cacheKey = `hover-${color}`;
  const cached = getCachedValue(cacheKey);
  if (cached) {
    return cached;
  }

  let backgroundColor: string;
  let boxShadow: string;

  try {
    backgroundColor = hexToRgba(color, 0.12);
  } catch (error) {
    console.warn('getHoverStyles: Failed to convert color to rgba for backgroundColor, using fallback:', error);
    try {
      backgroundColor = hexToRgba(sidebarTheme.colors.primary, 0.12);
    } catch {
      // Ultimate fallback with guaranteed valid color
      backgroundColor = 'rgba(25, 118, 210, 0.12)';
    }
  }

  try {
    boxShadow = `0 4px 12px ${hexToRgba(color, 0.19)}`;
  } catch (error) {
    console.warn('getHoverStyles: Failed to convert color to rgba for boxShadow, using fallback:', error);
    try {
      boxShadow = `0 4px 12px ${hexToRgba(sidebarTheme.colors.primary, 0.19)}`;
    } catch {
      // Ultimate fallback with guaranteed valid color
      boxShadow = '0 4px 12px rgba(25, 118, 210, 0.19)';
    }
  }

  const styles: SxProps<Theme> = {
    transition: sidebarTheme.transitions.hover,
    '&:hover': {
      backgroundColor,
      borderColor: color,
      transform: 'translateY(-1px)',
      boxShadow
    }
  };

  setCacheWithLimit(cacheKey, styles);
  return styles;
};

export const getFocusStyles = (color = sidebarTheme.colors.primary): SxProps<Theme> => {
  const cacheKey = `focus-${color}`;
  const cached = getCachedValue(cacheKey);
  if (cached) {
    return cached;
  }

  let boxShadow: string;

  try {
    boxShadow = `0 0 0 2px ${hexToRgba(color, 0.19)}`;
  } catch (error) {
    console.warn('getFocusStyles: Failed to convert color to rgba for boxShadow, using fallback:', error);
    try {
      boxShadow = `0 0 0 2px ${hexToRgba(sidebarTheme.colors.primary, 0.19)}`;
    } catch {
      // Ultimate fallback with guaranteed valid color
      boxShadow = '0 0 0 2px rgba(25, 118, 210, 0.19)';
    }
  }

  const styles: SxProps<Theme> = {
    '&.Mui-focused': {
      borderColor: color,
      boxShadow
    }
  };

  setCacheWithLimit(cacheKey, styles);
  return styles;
};

export const getSelectedStyles = (color = sidebarTheme.colors.primary): SxProps<Theme> => {
  const cacheKey = `selected-${color}`;
  const cached = getCachedValue(cacheKey);
  if (cached) {
    return cached;
  }

  let backgroundColor: string;

  try {
    backgroundColor = hexToRgba(color, 0.12);
  } catch (error) {
    console.warn('getSelectedStyles: Failed to convert color to rgba for backgroundColor, trying theme fallback:', error);
    try {
      backgroundColor = hexToRgba(sidebarTheme.colors.primary, 0.12);
    } catch (fallbackError) {
      console.warn('getSelectedStyles: Theme fallback color conversion also failed, using safe hardcoded fallback:', fallbackError);
      backgroundColor = 'rgba(0, 0, 0, 0.12)';
    }
  }

  const styles: SxProps<Theme> = {
    backgroundColor,
    borderColor: color,
    color
  };

  setCacheWithLimit(cacheKey, styles);
  return styles;
};
// Global button style override for all sidebar buttons
export const getSidebarButtonSx = (variant: 'primary' | 'secondary' | 'default' = 'default') => {
  return sidebarStyles.button[variant];
};

/**
 * Theme fragment for sidebar-specific button styling
 *
 * Usage: Wrap sidebar components with a ThemeProvider that merges this fragment:
 *
 * ```tsx
 * import { ThemeProvider, createTheme } from '@mui/material/styles';
 * import { sidebarButtonTheme } from './utils/sidebarStyles';
 *
 * const theme = createTheme({
 *   components: sidebarButtonTheme
 * });
 *
 * <ThemeProvider theme={theme}>
 *   <YourSidebarComponent />
 * </ThemeProvider>
 * ```
 *
 * This ensures button overrides only apply within the sidebar scope.
 */
export const sidebarButtonTheme = {
  MuiButton: {
    styleOverrides: {
      root: ({ theme }: { theme: Theme }) => ({
        borderRadius: sidebarTheme.radius.button,
        fontWeight: 500,
        textTransform: 'none',
        padding: '12px 24px',
        backgroundColor: sidebarTheme.colors.primary,
        color: sidebarTheme.colors.primaryContrast,
        border: `1px solid ${sidebarTheme.colors.primary}`,
        '&:hover': {
          backgroundColor: sidebarTheme.colors.hover,
          transform: 'translateY(-1px)',
          boxShadow: sidebarTheme.shadows.button
        },
        '&.MuiButton-outlined': {
          backgroundColor: 'transparent',
          borderColor: sidebarTheme.colors.primary,
          color: sidebarTheme.colors.primary,
          '&:hover': {
            backgroundColor: theme.palette.action.selected,
            borderColor: sidebarTheme.colors.primary
          }
        },
        '&.MuiButton-text': {
          backgroundColor: 'transparent',
          color: sidebarTheme.colors.primary,
          '&:hover': {
            backgroundColor: theme.palette.action.selected
          }
        }
      })
    }
  }
};