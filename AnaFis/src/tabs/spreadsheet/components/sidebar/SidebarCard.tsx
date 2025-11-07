import React, { useMemo } from 'react';
import { Paper, Box, Typography, SxProps, Theme } from '@mui/material';
import { sidebarTheme } from '@/tabs/spreadsheet/components/sidebar/themes/sidebarTheme';

// Static styles to prevent recreation
const BASE_PAPER_STYLES = {
  background: sidebarTheme.backgrounds.card,
  backdropFilter: 'blur(10px)',
  border: sidebarTheme.borders.card,
  borderRadius: sidebarTheme.radius.card,
  transition: sidebarTheme.transitions.default,
  '&:hover': {
    borderColor: 'rgba(33, 150, 243, 0.3)',
    boxShadow: sidebarTheme.shadows.cardHover,
    transform: 'translateY(-2px)'
  },
} as const;

const HEADER_STYLES = {
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'space-between',
} as const;

const ICON_BOX_STYLES = {
  display: 'flex',
  alignItems: 'center',
  gap: 1
} as const;

const ICON_STYLES = {
  color: sidebarTheme.colors.secondary,
  fontSize: '1.2rem'
} as const;

const COLLAPSE_ICON_STYLES = {
  transition: sidebarTheme.transitions.transform,
  color: sidebarTheme.colors.secondary,
  fontSize: '1.2rem'
} as const;

interface SidebarCardProps {
  title?: string;
  subtitle?: string;
  icon?: React.ReactNode;
  children: React.ReactNode;
  sx?: SxProps<Theme>;
  variant?: 'default' | 'compact' | 'elevated';
  collapsible?: boolean;
  defaultExpanded?: boolean;
}

export const SidebarCard = React.memo<SidebarCardProps>(({
  title,
  subtitle,
  icon,
  children,
  sx = {},
  variant = 'default',
  collapsible = false,
  defaultExpanded = true
}) => {
  const [expanded, setExpanded] = React.useState(defaultExpanded);

  // Memoized variant styles
  const variantStyles = useMemo(() => {
    switch (variant) {
      case 'compact':
        return {
          p: sidebarTheme.spacing.tight,
          mb: sidebarTheme.spacing.element
        };
      case 'elevated':
        return {
          p: sidebarTheme.spacing.section,
          mb: sidebarTheme.spacing.element,
          boxShadow: sidebarTheme.shadows.cardHover
        };
      default:
        return {
          p: sidebarTheme.spacing.section,
          mb: sidebarTheme.spacing.element
        };
    }
  }, [variant]);

  // Memoized combined styles
  const combinedStyles = useMemo(() => ({
    ...BASE_PAPER_STYLES,
    ...variantStyles,
    ...sx
  }), [variantStyles, sx]);

  // Memoized header styles
  const headerBoxStyles = useMemo(() => ({
    ...HEADER_STYLES,
    mb: title ? sidebarTheme.spacing.element : 0,
    cursor: collapsible ? 'pointer' : 'default'
  }), [title, collapsible]);

  // Memoized collapse icon styles
  const collapseIconStyles = useMemo(() => ({
    ...COLLAPSE_ICON_STYLES,
    transform: expanded ? 'rotate(180deg)' : 'rotate(0deg)',
  }), [expanded]);

  return (
    <Paper
      sx={combinedStyles}
      elevation={0}
    >
      {/* Header */}
      {(title ?? icon) && (
        <Box
          sx={headerBoxStyles}
          onClick={collapsible ? () => setExpanded(prev => !prev) : undefined}
          onKeyDown={collapsible ? (event) => {
            if (event.key === 'Enter' || event.key === ' ') {
              event.preventDefault();
              setExpanded(prev => !prev);
            }
          } : undefined}
          role={collapsible ? 'button' : undefined}
          tabIndex={collapsible ? 0 : undefined}
          aria-expanded={collapsible ? expanded : undefined}
        >
          <Box sx={ICON_BOX_STYLES}>
            {icon && (
              <Box sx={ICON_STYLES}>
                {icon}
              </Box>
            )}
            {title && (
              <Typography sx={sidebarTheme.typography.sectionTitle}>
                {title}
              </Typography>
            )}
          </Box>

          {collapsible && (
            <Box sx={collapseIconStyles} aria-hidden="true">
              ▼
            </Box>
          )}
        </Box>
      )}

      {/* Subtitle */}
      {subtitle && (
        <Typography
          sx={{
            ...sidebarTheme.typography.caption,
            mb: sidebarTheme.spacing.element,
            color: sidebarTheme.colors.text.TERTIARY
          }}
        >
          {subtitle}
        </Typography>
      )}

      {/* Content */}
      {(!collapsible || expanded) && (
        <Box>
          {children}
        </Box>
      )}
    </Paper>
  );
});

SidebarCard.displayName = 'SidebarCard';

export default SidebarCard;