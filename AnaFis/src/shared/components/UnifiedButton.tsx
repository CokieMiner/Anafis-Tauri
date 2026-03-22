/**
 * Unified Button Component
 * Single source of truth for all button styles across AnaFis
 *
 * @example
 * // Primary button
 * <UnifiedButton variant="primary">Click me</UnifiedButton>
 *
 * // Tab button with color
 * <UnifiedButton variant="tab" color="spreadsheet">New Spreadsheet</UnifiedButton>
 *
 * // Window control button
 * <UnifiedButton variant="windowControl" control="close" />
 */

import {
  Button,
  type ButtonProps,
  IconButton,
  type IconButtonProps,
} from '@mui/material';
import { forwardRef, useMemo } from 'react';
import { anafisTheme, type TabType } from '@/shared/theme/unifiedTheme';

// =============================================================================
// TYPES
// =============================================================================

export type ButtonVariant =
  | 'primary'
  | 'secondary'
  | 'tab'
  | 'windowControl'
  | 'icon';
export type WindowControlType = 'minimize' | 'maximize' | 'close';

export interface UnifiedButtonProps
  extends Omit<ButtonProps, 'variant' | 'color'> {
  variant?: ButtonVariant;
  /** Tab type for 'tab' variant */
  tabType?: TabType;
  /** Window control type for 'windowControl' variant */
  control?: WindowControlType;
  /** Hover color override (for custom styling) */
  hoverColor?: string;
  /** Hover background color override */
  hoverBackgroundColor?: string;
  /** Hover border color override */
  hoverBorderColor?: string;
  /** Hover box shadow color override */
  hoverBoxShadowColor?: string;
}

export interface UnifiedIconButtonProps extends Omit<IconButtonProps, 'color'> {
  control?: WindowControlType;
  tabType?: TabType;
}

// =============================================================================
// STYLE GENERATORS
// =============================================================================

interface TabButtonConfig {
  hoverColor: string;
  hoverBackgroundColor: string;
  hoverBorderColor: string;
  hoverBoxShadowColor: string;
}

/**
 * Get tab-specific button configuration
 */
function getTabButtonConfig(tabType: TabType): TabButtonConfig {
  const tabColor = anafisTheme.colors.tabs[tabType];
  return {
    hoverColor: tabColor.light,
    hoverBackgroundColor: `${tabColor.main}20`,
    hoverBorderColor: `${tabColor.main}33`,
    hoverBoxShadowColor: `${tabColor.main}4D`,
  };
}

/**
 * Get window control button styles
 */
function getWindowControlStyles(control: WindowControlType) {
  const colors = anafisTheme.colors.windowControls;
  const controlColors = {
    minimize: colors.minimize,
    maximize: colors.maximize,
    close: colors.close,
  };

  const color = controlColors[control];

  return {
    width: '32px',
    height: '32px',
    borderRadius: 0,
    color: 'rgba(255, 255, 255, 0.8)',
    backgroundColor: 'transparent',
    border: 'none',
    outline: 'none',
    boxShadow: 'none',
    transition: anafisTheme.transitions.hover,
    '&:hover': {
      backgroundColor: `${color.main} !important`,
      color: '#ffffff',
      transform: 'scale(1.1)',
      boxShadow: `0 2px 8px ${color.main}66`,
      outline: 'none !important',
      border: 'none !important',
    },
    '&:active': {
      backgroundColor: `${color.dark} !important`,
      transform: 'scale(0.95)',
      outline: 'none !important',
      border: 'none !important',
      boxShadow: 'none !important',
    },
    '&:focus': {
      outline: 'none !important',
      border: 'none !important',
      boxShadow: 'none !important',
    },
    '&.Mui-focusVisible': {
      outline: 'none !important',
      border: 'none !important',
      boxShadow: 'none !important',
    },
    '&.Mui-disabled': {
      color: 'rgba(255, 255, 255, 0.3)',
    },
  };
}

// =============================================================================
// UNIFIED BUTTON COMPONENT
// =============================================================================

export const UnifiedButton = forwardRef<HTMLButtonElement, UnifiedButtonProps>(
  (
    {
      variant = 'primary',
      tabType = 'spreadsheet',
      control = 'close',
      hoverColor,
      hoverBackgroundColor,
      hoverBorderColor,
      hoverBoxShadowColor,
      children,
      sx,
      ...props
    },
    ref
  ) => {
    // Generate styles based on variant
    const buttonStyles = useMemo(() => {
      const baseStyles = {
        borderRadius: anafisTheme.radius.button,
        textTransform: 'none' as const,
        transition: anafisTheme.transitions.hover,
      };

      switch (variant) {
        case 'primary': {
          return {
            ...baseStyles,
            fontWeight: 600,
            backgroundColor: anafisTheme.colors.primary.main,
            color: anafisTheme.colors.primary.contrast,
            border: `1px solid ${anafisTheme.colors.primary.main}`,
            px: 2,
            py: 1,
            '&:hover': {
              backgroundColor: anafisTheme.colors.primary.dark,
              transform: 'translateY(-1px)',
              boxShadow: anafisTheme.shadows.primary,
            },
            '&:disabled': {
              backgroundColor: anafisTheme.colors.sidebar.disabledBg,
              color: anafisTheme.colors.sidebar.disabledText,
            },
          };
        }

        case 'secondary': {
          return {
            ...baseStyles,
            fontWeight: 500,
            backgroundColor: 'transparent',
            color: anafisTheme.colors.primary.main,
            border: `1px solid ${anafisTheme.colors.primary.main}`,
            px: 2,
            py: 1,
            '&:hover': {
              borderColor: anafisTheme.colors.primary.main,
              backgroundColor: `${anafisTheme.colors.primary.main}1A`,
              transform: 'translateY(-1px)',
            },
          };
        }

        case 'tab': {
          const tabConfig = getTabButtonConfig(tabType);
          return {
            ...baseStyles,
            color: anafisTheme.colors.text.secondary,
            borderRadius: anafisTheme.radius.lg,
            px: 2,
            '&:hover': {
              backgroundColor:
                hoverBackgroundColor ?? tabConfig.hoverBackgroundColor,
              color: hoverColor ?? tabConfig.hoverColor,
              transform: 'translateY(-1px)',
              boxShadow: `0 4px 12px ${hoverBoxShadowColor ?? tabConfig.hoverBoxShadowColor}`,
              border: `1px solid ${hoverBorderColor ?? tabConfig.hoverBorderColor}`,
            },
            '&:active': {
              transform: 'translateY(0px)',
            },
          };
        }

        case 'windowControl': {
          return getWindowControlStyles(control);
        }

        default:
          return baseStyles;
      }
    }, [
      variant,
      tabType,
      control,
      hoverColor,
      hoverBackgroundColor,
      hoverBorderColor,
      hoverBoxShadowColor,
    ]);

    return (
      <Button ref={ref} sx={{ ...buttonStyles, ...sx }} {...props}>
        {children}
      </Button>
    );
  }
);

UnifiedButton.displayName = 'UnifiedButton';

// =============================================================================
// UNIFIED ICON BUTTON COMPONENT
// =============================================================================

export const UnifiedIconButton = forwardRef<
  HTMLButtonElement,
  UnifiedIconButtonProps
>(({ control, tabType, sx, ...props }, ref) => {
  const iconButtonStyles = useMemo(() => {
    if (control) {
      return getWindowControlStyles(control);
    }

    // Default icon button styles
    const color = tabType
      ? anafisTheme.colors.tabs[tabType].main
      : anafisTheme.colors.primary.main;

    return {
      color: anafisTheme.colors.text.secondary,
      borderRadius: anafisTheme.radius.button,
      transition: anafisTheme.transitions.hover,
      '&:hover': {
        backgroundColor: `${color}1A`,
        color: anafisTheme.colors.text.primary,
        transform: 'scale(1.05)',
      },
    };
  }, [control, tabType]);

  return (
    <IconButton ref={ref} sx={{ ...iconButtonStyles, ...sx }} {...props} />
  );
});

UnifiedIconButton.displayName = 'UnifiedIconButton';

// =============================================================================
// PRE-CONFIGURED BUTTON EXPORTS
// =============================================================================

/** Pre-configured minimize button for window controls */
export const MinimizeButton = forwardRef<
  HTMLButtonElement,
  Omit<UnifiedIconButtonProps, 'control'>
>((props, ref) => (
  <UnifiedIconButton ref={ref} control="minimize" {...props} />
));

/** Pre-configured maximize button for window controls */
export const MaximizeButton = forwardRef<
  HTMLButtonElement,
  Omit<UnifiedIconButtonProps, 'control'>
>((props, ref) => (
  <UnifiedIconButton ref={ref} control="maximize" {...props} />
));

/** Pre-configured close button for window controls */
export const CloseButton = forwardRef<
  HTMLButtonElement,
  Omit<UnifiedIconButtonProps, 'control'>
>((props, ref) => <UnifiedIconButton ref={ref} control="close" {...props} />);

MinimizeButton.displayName = 'MinimizeButton';
MaximizeButton.displayName = 'MaximizeButton';
CloseButton.displayName = 'CloseButton';

// =============================================================================
// TAB BUTTON COMPONENTS
// =============================================================================

/** Pre-configured spreadsheet tab button */
export const SpreadsheetTabButton = forwardRef<
  HTMLButtonElement,
  Omit<UnifiedButtonProps, 'variant' | 'tabType'>
>((props, ref) => (
  <UnifiedButton ref={ref} variant="tab" tabType="spreadsheet" {...props} />
));

/** Pre-configured fitting tab button */
export const FittingTabButton = forwardRef<
  HTMLButtonElement,
  Omit<UnifiedButtonProps, 'variant' | 'tabType'>
>((props, ref) => (
  <UnifiedButton ref={ref} variant="tab" tabType="fitting" {...props} />
));

/** Pre-configured solver tab button */
export const SolverTabButton = forwardRef<
  HTMLButtonElement,
  Omit<UnifiedButtonProps, 'variant' | 'tabType'>
>((props, ref) => (
  <UnifiedButton ref={ref} variant="tab" tabType="solver" {...props} />
));

/** Pre-configured monte carlo tab button */
export const MonteCarloTabButton = forwardRef<
  HTMLButtonElement,
  Omit<UnifiedButtonProps, 'variant' | 'tabType'>
>((props, ref) => (
  <UnifiedButton ref={ref} variant="tab" tabType="montecarlo" {...props} />
));

SpreadsheetTabButton.displayName = 'SpreadsheetTabButton';
FittingTabButton.displayName = 'FittingTabButton';
SolverTabButton.displayName = 'SolverTabButton';
MonteCarloTabButton.displayName = 'MonteCarloTabButton';
