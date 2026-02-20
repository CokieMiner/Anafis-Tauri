import type { SxProps, Theme } from '@mui/material';
import { sidebarTheme } from '@/tabs/spreadsheet/components/sidebar/themes/sidebarTheme';

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
  backdropFilter: 'blur(20px)',
};

const HEADER_STYLE: SxProps<Theme> = {
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'space-between',
  p: sidebarTheme.spacing.container,
  background: sidebarTheme.backgrounds.header,
  borderBottom: sidebarTheme.borders.section,
  backdropFilter: 'blur(10px)',
};

const CONTENT_STYLE: SxProps<Theme> = {
  flex: 1,
  overflow: 'auto',
  p: sidebarTheme.spacing.container,
  background: sidebarTheme.backgrounds.section,
};

// Content wrapper for scrollable content areas
const CONTENT_WRAPPER_STYLE: SxProps<Theme> = {
  flex: 1,
  display: 'flex',
  flexDirection: 'column',
  overflow: 'auto', // Changed from 'hidden' to 'auto' to enable scrolling
  gap: sidebarTheme.spacing.element,
  p: sidebarTheme.spacing.container,
};

const FOOTER_STYLE: SxProps<Theme> = {
  p: sidebarTheme.spacing.container,
  borderTop: sidebarTheme.borders.section,
  background: sidebarTheme.backgrounds.header,
  display: 'flex',
  gap: sidebarTheme.spacing.element,
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
    mb: sidebarTheme.spacing.element,
  } as SxProps<Theme>,

  // Form section
  formSection: {
    mb: sidebarTheme.spacing.section,
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
        boxShadow: sidebarTheme.shadows.button,
      },
      '&:disabled': {
        backgroundColor: sidebarTheme.colors.disabledBg,
        color: sidebarTheme.colors.disabledText,
      },
    } as const;

    return {
      primary: {
        ...baseButton,
        fontWeight: 600,
        backgroundColor: sidebarTheme.colors.primary,
        color: sidebarTheme.colors.primaryContrast,
        border: `1px solid ${sidebarTheme.colors.primary}`,
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
          transform: 'translateY(-1px)',
        },
      } as SxProps<Theme>,

      // Default button style that should be applied to all buttons
      default: {
        ...baseButton,
        fontWeight: 500,
        backgroundColor: sidebarTheme.colors.primary,
        color: sidebarTheme.colors.primaryContrast,
        border: `1px solid ${sidebarTheme.colors.primary}`,
      } as SxProps<Theme>,
    };
  })(),

  // Input styles
  input: {
    ...sidebarTheme.components.input.root,
    mb: sidebarTheme.spacing.element,
  } as SxProps<Theme>,

  // Typography styles
  text: {
    header: sidebarTheme.typography.header,
    sectionTitle: sidebarTheme.typography.sectionTitle,
    label: sidebarTheme.typography.label,
    caption: sidebarTheme.typography.caption,
    body: sidebarTheme.typography.body,
  },

  // Layout helpers
  flexColumn: {
    display: 'flex',
    flexDirection: 'column',
  } as SxProps<Theme>,

  flexRow: {
    display: 'flex',
    alignItems: 'center',
  } as SxProps<Theme>,

  spaceBetween: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
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
      overflow: 'auto',
    } as SxProps<Theme>,
    rightColumn: {
      flex: 1,
      overflow: 'auto',
    } as SxProps<Theme>,
  },

  // Icon button styles
  iconButton: {
    color: sidebarTheme.colors.text.SECONDARY,
    borderRadius: sidebarTheme.radius.button,
    transition: sidebarTheme.transitions.hover,
    '&:hover': {
      backgroundColor: sidebarTheme.backgrounds.hover,
      color: sidebarTheme.colors.text.PRIMARY,
      transform: 'scale(1.05)',
    },
  } as SxProps<Theme>,
};
