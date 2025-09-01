import React from 'react';
import { Button } from '@mui/material';
import { AddIcon } from '../icons';

interface TabButtonProps {
  label: string;
  onClick: () => void;
  hoverColor: string;
  hoverBackgroundColor: string;
  hoverBorderColor: string;
  hoverBoxShadowColor: string;
}

const TabButton: React.FC<TabButtonProps> = ({
  label,
  onClick,
  hoverColor,
  hoverBackgroundColor,
  hoverBorderColor,
  hoverBoxShadowColor,
}) => {
  return (
    <Button
      color="inherit"
      startIcon={<AddIcon />}
      onClick={onClick}
      sx={{
        color: 'text.secondary',
        borderRadius: 2,
        px: 2,
        transition: 'all 0.2s ease-in-out',
        '&:hover': {
          backgroundColor: hoverBackgroundColor,
          color: hoverColor,
          transform: 'translateY(-1px)',
          boxShadow: `0 4px 12px ${hoverBoxShadowColor}`,
          border: `1px solid ${hoverBorderColor}`,
        },
        '&:active': {
          transform: 'translateY(0px)',
        },
      }}
    >
      {label}
    </Button>
  );
};

export default TabButton;
