import React from 'react';
import { BooleanCell as BooleanCellData } from '../../../types/spreadsheet';

interface BooleanCellProps {
  cellData: BooleanCellData | null;
  onChange: (newCellData: BooleanCellData) => void;
}

export const BooleanCellEditor: React.FC<BooleanCellProps> = ({
  cellData,
  onChange,
}) => {
  return (
    <input
      type="checkbox"
      autoFocus
      defaultChecked={cellData?.value ?? false}
      onChange={(e) => {
        onChange({ type: 'boolean', value: e.target.checked });
      }}
    />
  );
};
