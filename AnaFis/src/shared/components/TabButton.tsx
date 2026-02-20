import { Button } from '@mui/material';
import React, { useMemo } from 'react';
import { AddIcon } from '@/icons';

// Base styles to prevent recreation
const BASE_BUTTON_STYLES = {
  color: 'text.secondary',
  borderRadius: 2,
  px: 2,
  transition: 'all 0.2s ease-in-out',
  '&:active': {
    transform: 'translateY(0px)',
  },
} as const;

interface TabButtonProps {
  label: string;
  onClick: () => void;
  hoverColor: string;
  hoverBackgroundColor: string;
  hoverBorderColor: string;
  hoverBoxShadowColor: string;
}

const TabButton = React.memo<TabButtonProps>(
  ({
    label,
    onClick,
    hoverColor,
    hoverBackgroundColor,
    hoverBorderColor,
    hoverBoxShadowColor,
  }) => {
    // Memoized dynamic styles
    const buttonStyles = useMemo(
      () => ({
        ...BASE_BUTTON_STYLES,
        '&:hover': {
          backgroundColor: hoverBackgroundColor,
          color: hoverColor,
          transform: 'translateY(-1px)',
          boxShadow: `0 4px 12px ${hoverBoxShadowColor}`,
          border: `1px solid ${hoverBorderColor}`,
        },
      }),
      [hoverColor, hoverBackgroundColor, hoverBorderColor, hoverBoxShadowColor]
    );

    return (
      <Button
        color="inherit"
        startIcon={<AddIcon />}
        onClick={onClick}
        sx={buttonStyles}
      >
        {label}
      </Button>
    );
  }
);

TabButton.displayName = 'TabButton';

export default TabButton;
