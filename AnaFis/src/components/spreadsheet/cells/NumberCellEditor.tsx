import React from 'react';
import { NumberCell as NumberCellData } from '../../../types/spreadsheet';

interface NumberCellProps {
  cellData: NumberCellData | null;
  onChange: (newCellData: NumberCellData) => void;
}

export const NumberCellEditor: React.FC<NumberCellProps> = ({
  cellData,
  onChange,
}) => {
  return (
    <input
      type="number"
      autoFocus
      defaultValue={cellData?.value ?? ''}
      onChange={(e) => {
        onChange({ type: 'number', value: parseFloat(e.target.value) });
      }}
    />
  );
};
