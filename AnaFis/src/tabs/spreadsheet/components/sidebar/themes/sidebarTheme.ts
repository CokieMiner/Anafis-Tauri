/**
 * AnaFis Sidebar Theme - Re-exports from unified theme system
 * This file maintains backwards compatibility while using the unified theme
 *
 * @deprecated Import from '@/shared/theme/unifiedTheme' instead
 */

import { anafisTheme } from '@/shared/theme/unifiedTheme';

// Re-export for backwards compatibility
export const sidebarTheme = {
  // Color Palette - derived from unified theme
  colors: {
    primary: anafisTheme.colors.sidebar.primary,
    primaryDark: anafisTheme.colors.sidebar.primaryDark,
    primaryContrast: anafisTheme.colors.sidebar.primaryContrast,
    secondary: anafisTheme.colors.sidebar.secondary,
    accent: anafisTheme.colors.sidebar.accent,
    success: anafisTheme.colors.status.success.main,
    warning: anafisTheme.colors.status.warning.main,
    error: anafisTheme.colors.status.error.main,
    hover: anafisTheme.colors.sidebar.hover,
    disabledBg: anafisTheme.colors.sidebar.disabledBg,
    disabledText: anafisTheme.colors.sidebar.disabledText,
    text: {
      PRIMARY: anafisTheme.colors.text.primary,
      SECONDARY: anafisTheme.colors.text.secondary,
      TERTIARY: anafisTheme.colors.text.tertiary,
      DISABLED: anafisTheme.colors.text.disabled,
    },
  },

  // Background Gradients - from unified theme
  backgrounds: {
    container: anafisTheme.gradients.backgroundRadial,
    header: anafisTheme.gradients.backgroundRadial,
    card: anafisTheme.gradients.card,
    section: `${anafisTheme.colors.sidebar.primary}0D`, // 0.05 opacity
    hover: `${anafisTheme.colors.sidebar.primary}1A`, // 0.1 opacity
  },

  // Border Styles
  borders: {
    container: `1px solid ${anafisTheme.colors.sidebar.primary}4D`,
    leftAccent: `3px solid ${anafisTheme.colors.sidebar.primary}`,
    section: `1px solid ${anafisTheme.colors.sidebar.primary}33`,
    card: `1px solid ${anafisTheme.colors.sidebar.primary}1A`,
    focus: `1px solid ${anafisTheme.colors.sidebar.primary}`,
  },

  // Shadow Effects - from unified theme
  shadows: {
    container: anafisTheme.shadows.container,
    card: anafisTheme.shadows.card,
    cardHover: anafisTheme.shadows.cardHover,
    button: anafisTheme.shadows.primary,
  },

  // Typography Scale - from unified theme
  typography: {
    header: {
      ...anafisTheme.typography.header,
      color: anafisTheme.colors.sidebar.primary,
    },
    sectionTitle: {
      ...anafisTheme.typography.sectionTitle,
      color: anafisTheme.colors.sidebar.secondary,
    },
    label: {
      ...anafisTheme.typography.label,
      color: anafisTheme.colors.text.primary,
    },
    caption: {
      ...anafisTheme.typography.caption,
      color: anafisTheme.colors.text.tertiary,
    },
    body: {
      ...anafisTheme.typography.body,
      color: anafisTheme.colors.text.primary,
    },
  },

  // Spacing Scale - from unified theme
  spacing: {
    container: anafisTheme.spacing.xl, // 20px
    section: anafisTheme.spacing.lg, // 16px
    element: anafisTheme.spacing.md, // 12px
    tight: anafisTheme.spacing.sm, // 8px
    loose: anafisTheme.spacing.xxl, // 24px
  },

  // Border Radius - from unified theme
  radius: {
    container: anafisTheme.radius.container,
    card: anafisTheme.radius.card,
    button: anafisTheme.radius.button,
    input: anafisTheme.radius.input,
  },

  // Transitions - from unified theme
  transitions: {
    default: anafisTheme.transitions.default,
    hover: anafisTheme.transitions.hover,
    focus: anafisTheme.transitions.focus,
    transform: anafisTheme.transitions.transform,
  },

  // Component-specific styles
  components: {
    button: {
      primary: {
        background: anafisTheme.gradients.buttonPrimary,
        color: anafisTheme.colors.sidebar.primaryContrast,
        '&:hover': {
          background: anafisTheme.gradients.buttonPrimaryHover,
          transform: 'translateY(-1px)',
          boxShadow: anafisTheme.shadows.primary,
        },
      },
      secondary: {
        borderColor: `${anafisTheme.colors.sidebar.primary}80`,
        color: anafisTheme.colors.sidebar.secondary,
        '&:hover': {
          borderColor: anafisTheme.colors.sidebar.primary,
          backgroundColor: `${anafisTheme.colors.sidebar.primary}1A`,
          transform: 'translateY(-1px)',
        },
      },
      default: {
        backgroundColor: anafisTheme.colors.sidebar.primary,
        color: anafisTheme.colors.sidebar.primaryContrast,
        border: 'none',
        '&:hover': {
          backgroundColor: anafisTheme.colors.sidebar.hover,
          transform: 'translateY(-1px)',
          boxShadow: anafisTheme.shadows.primary,
        },
      },
    },
    input: {
      root: {
        '& .MuiOutlinedInput-root': {
          backgroundColor: `${anafisTheme.colors.sidebar.primary}0D`,
          borderRadius: anafisTheme.radius.input,
          '& fieldset': {
            borderColor: `${anafisTheme.colors.sidebar.primary}33`,
          },
          '&:hover fieldset': {
            borderColor: `${anafisTheme.colors.sidebar.primary}66`,
          },
          '&.Mui-focused fieldset': {
            borderColor: anafisTheme.colors.sidebar.primary,
          },
        },
        '& .MuiInputLabel-root': {
          color: anafisTheme.colors.sidebar.secondary,
          '&.Mui-focused': {
            color: anafisTheme.colors.sidebar.primary,
          },
        },
        '& input': {
          color: anafisTheme.colors.text.primary,
        },
      },
    },
  },
} as const;

// Type exports for TypeScript
export type SidebarTheme = typeof sidebarTheme;
export type SidebarColors = typeof sidebarTheme.colors;
export type SidebarTypography = typeof sidebarTheme.typography;
