import React from 'react';
import { Paper, Box, Typography, SxProps, Theme } from '@mui/material';
import { sidebarTheme } from '../../themes/sidebarTheme';

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

export const SidebarCard: React.FC<SidebarCardProps> = ({
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

  const getVariantStyles = () => {
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
  };

  return (
    <Paper
      sx={{
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
        ...getVariantStyles(),
        ...sx
      }}
      elevation={0}
    >
      {/* Header */}
      {(title || icon) && (
        <Box
          sx={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            mb: title ? sidebarTheme.spacing.element : 0,
            cursor: collapsible ? 'pointer' : 'default'
          }}
          onClick={collapsible ? () => setExpanded(!expanded) : undefined}
        >
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            {icon && (
              <Box sx={{ color: sidebarTheme.colors.secondary, fontSize: '1.2rem' }}>
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
            <Box
              sx={{
                transform: expanded ? 'rotate(180deg)' : 'rotate(0deg)',
                transition: sidebarTheme.transitions.transform,
                color: sidebarTheme.colors.secondary,
                fontSize: '1.2rem'
              }}
            >
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
            color: sidebarTheme.colors.text.tertiary
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
};

export default SidebarCard;