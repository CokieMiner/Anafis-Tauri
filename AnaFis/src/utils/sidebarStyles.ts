import { SxProps, Theme } from '@mui/material';
import { sidebarTheme } from '../themes/sidebarTheme';

// Shared styling utilities for consistent sidebar design

export const sidebarStyles = {
  // Container styles
  container: {
    width: 450,
    height: '100%',
    display: 'flex',
    flexDirection: 'column',
    background: sidebarTheme.backgrounds.container,
    border: sidebarTheme.borders.container,
    borderLeft: sidebarTheme.borders.leftAccent,
    borderRadius: sidebarTheme.radius.container,
    overflow: 'hidden',
    boxShadow: sidebarTheme.shadows.container,
    backdropFilter: 'blur(20px)'
  } as SxProps<Theme>,

  // Header styles
  header: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    p: sidebarTheme.spacing.container,
    background: sidebarTheme.backgrounds.header,
    borderBottom: sidebarTheme.borders.section,
    backdropFilter: 'blur(10px)'
  } as SxProps<Theme>,

  // Content area
  content: {
    flex: 1,
    overflow: 'auto',
    p: sidebarTheme.spacing.container,
    background: sidebarTheme.backgrounds.section
  } as SxProps<Theme>,

  // Footer styles
  footer: {
    p: sidebarTheme.spacing.container,
    borderTop: sidebarTheme.borders.section,
    background: sidebarTheme.backgrounds.header,
    display: 'flex',
    gap: sidebarTheme.spacing.element
  } as SxProps<Theme>,

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

  // Button styles
  button: {
    primary: {
      ...sidebarTheme.components.button.primary,
      borderRadius: sidebarTheme.radius.button,
      fontWeight: 600,
      textTransform: 'none',
      px: 3,
      py: 1.5
    } as SxProps<Theme>,

    secondary: {
      ...sidebarTheme.components.button.secondary,
      borderRadius: sidebarTheme.radius.button,
      fontWeight: 500,
      textTransform: 'none',
      px: 3,
      py: 1.5
    } as SxProps<Theme>
  },

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
    color: sidebarTheme.colors.text.secondary,
    borderRadius: sidebarTheme.radius.button,
    transition: sidebarTheme.transitions.hover,
    '&:hover': {
      backgroundColor: sidebarTheme.backgrounds.hover,
      color: sidebarTheme.colors.text.primary,
      transform: 'scale(1.05)'
    }
  } as SxProps<Theme>
};

// Utility functions for dynamic styles
export const getHoverStyles = (color = sidebarTheme.colors.primary) => ({
  transition: sidebarTheme.transitions.hover,
  '&:hover': {
    backgroundColor: `${color}20`,
    borderColor: color,
    transform: 'translateY(-1px)',
    boxShadow: `0 4px 12px ${color}30`
  }
});

export const getFocusStyles = (color = sidebarTheme.colors.primary) => ({
  '&.Mui-focused': {
    borderColor: color,
    boxShadow: `0 0 0 2px ${color}30`
  }
});

export const getSelectedStyles = (color = sidebarTheme.colors.primary) => ({
  backgroundColor: `${color}20`,
  borderColor: color,
  color: color
});