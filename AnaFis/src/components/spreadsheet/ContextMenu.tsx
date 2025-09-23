import React from 'react';
import { Menu, MenuItem } from '@mui/material';
import { ContextMenuComponentProps } from '@sdziadkowiec/react-datasheet-grid';
import { CellType } from '../../types/spreadsheet';

interface CustomContextMenuProps extends ContextMenuComponentProps {
  onCellTypeChange: (cellType: CellType) => void;
}

export const ContextMenu: React.FC<CustomContextMenuProps> = ({
  clientX,
  clientY,
  close,
  onCellTypeChange,
}) => {
  const handleTypeChange = (cellType: CellType) => {
    onCellTypeChange(cellType);
    close();
  };

  return (
    <Menu
      open={true}
      onClose={close}
      anchorReference="anchorPosition"
      anchorPosition={{ top: clientY, left: clientX }}
    >
      <MenuItem onClick={() => handleTypeChange('text')}>Change to Text</MenuItem>
      <MenuItem onClick={() => handleTypeChange('number')}>Change to Number</MenuItem>
      <MenuItem onClick={() => handleTypeChange('boolean')}>Change to Boolean</MenuItem>
      <MenuItem onClick={() => handleTypeChange('uncertainty')}>Change to Uncertainty</MenuItem>
    </Menu>
  );
};
