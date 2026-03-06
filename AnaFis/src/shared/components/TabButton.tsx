import React from 'react';
import { AddIcon } from '@/icons';
import { UnifiedButton } from '@/shared/components/UnifiedButton';

interface TabButtonProps {
  label: string;
  onClick: () => void;
  hoverColor: string;
  hoverBackgroundColor: string;
  hoverBorderColor: string;
  hoverBoxShadowColor: string;
}

const TabButton = React.memo<TabButtonProps>(
  ({ label, onClick, ...hoverProps }) => {
    return (
      <UnifiedButton
        variant="tab"
        startIcon={<AddIcon />}
        onClick={onClick}
        {...hoverProps}
      >
        {label}
      </UnifiedButton>
    );
  }
);

TabButton.displayName = 'TabButton';

export default TabButton;
