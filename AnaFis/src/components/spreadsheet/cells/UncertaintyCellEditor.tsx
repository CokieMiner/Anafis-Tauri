import React from 'react';
import { UncertaintyCell as UncertaintyCellData, Uncertainty } from '../../../types/spreadsheet';

interface UncertaintyCellProps {
  cellData: UncertaintyCellData | null;
  onChange: (newCellData: UncertaintyCellData) => void;
}

export const UncertaintyCellEditor: React.FC<UncertaintyCellProps> = ({
  cellData,
  onChange,
}) => {
  const value = cellData?.value?.value ?? '';
  const uncertainty = cellData?.value?.uncertainty ?? '';

  const handleChange = (newValue: Partial<Uncertainty>) => {
    const newUncertainty: Uncertainty = {
      value: cellData?.value?.value ?? null,
      uncertainty: cellData?.value?.uncertainty ?? null,
      ...newValue,
    };
    onChange({ type: 'uncertainty', value: newUncertainty });
  };

  return (
    <div>
      <input
        type="number"
        autoFocus
        defaultValue={value}
        onChange={(e) => handleChange({ value: parseFloat(e.target.value) })}
      />
      <span> ± </span>
      <input
        type="number"
        defaultValue={uncertainty}
        onChange={(e) => handleChange({ uncertainty: parseFloat(e.target.value) })}
      />
    </div>
  );
};
