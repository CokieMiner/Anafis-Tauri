import React from 'react';
import { Column } from '@sdziadkowiec/react-datasheet-grid';
import { SpreadsheetRow, Cell } from '../../types/spreadsheet';
import { TextCellEditor } from './cells/TextCellEditor';
import { NumberCellEditor } from './cells/NumberCellEditor';
import { BooleanCellEditor } from './cells/BooleanCellEditor';
import { UncertaintyCellEditor } from './cells/UncertaintyCellEditor';

interface UniversalColumnProps {
  handleCellUpdate: (rowIndex: number, columnIndex: number, newCellData: Cell) => void;
}

export const createUniversalColumn = ({ handleCellUpdate }: UniversalColumnProps): Partial<Column<SpreadsheetRow>> => ({
  component: ({ rowData, rowIndex, columnIndex, onChange }) => {
    const cellData = rowData[columnIndex];

    const handleCellChange = (newCellData: Cell) => {
      handleCellUpdate(rowIndex, columnIndex, newCellData);
      const newRowData = [...rowData];
      newRowData[columnIndex] = newCellData;
      onChange(newRowData, true);
    };

    switch (cellData?.type) {
      case 'text':
        return <TextCellEditor cellData={cellData} onChange={handleCellChange} />;
      case 'number':
        return <NumberCellEditor cellData={cellData} onChange={handleCellChange} />;
      case 'boolean':
        return <BooleanCellEditor cellData={cellData} onChange={handleCellChange} />;
      case 'uncertainty':
        return <UncertaintyCellEditor cellData={cellData} onChange={handleCellChange} />;
      default:
        return <TextCellEditor cellData={null} onChange={handleCellChange} />;
    }
  },
  render: ({ rowData, columnIndex }) => {
    const cellData = rowData[columnIndex];

    switch (cellData?.type) {
      case 'text':
        return <div>{cellData.value}</div>;
      case 'number':
        return <div>{cellData.value}</div>;
      case 'boolean':
        return <input type="checkbox" checked={cellData.value ?? false} disabled />;
      case 'uncertainty':
        return <div>{cellData.value ? `${cellData.value.value} ± ${cellData.value.uncertainty}` : ''}</div>;
      default:
        return <div />;
    }
  },
});
