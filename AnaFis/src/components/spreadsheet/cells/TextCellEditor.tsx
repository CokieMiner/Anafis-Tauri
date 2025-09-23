import React from 'react';
import { TextCell as TextCellData } from '../../../types/spreadsheet';

interface TextCellProps {
  cellData: TextCellData | null;
  onChange: (newCellData: TextCellData) => void;
}

export const TextCellEditor: React.FC<TextCellProps> = ({
  cellData,
  onChange,
}) => {
  return (
    <input
      autoFocus
      defaultValue={cellData?.value ?? ''}
      onChange={(e) => {
        onChange({ type: 'text', value: e.target.value });
      }}
    />
  );
};
