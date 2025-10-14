// UniverAdapter.tsx - Wraps UniverSpreadsheet to match abstract interface
import { forwardRef, useImperativeHandle, useRef, useMemo } from 'react';
import { IWorkbookData, ICellData, LocaleType } from '@univerjs/core';
import UniverSpreadsheet, { UniverSpreadsheetRef as OriginalUniverRef } from './UniverSpreadsheet';
import { SpreadsheetProps, SpreadsheetRef, CellValue, WorkbookData } from '../SpreadsheetInterface';

// Convert abstract WorkbookData to Univer IWorkbookData
const convertToUniverData = (data: WorkbookData): IWorkbookData => {
  const sheetId = Object.keys(data.sheets)[0] || 'sheet-01';
  const sheet = data.sheets[sheetId];

  // Convert cellData from Record<string, CellValue> to Record<string, ICellData>
  const cellData: Record<string, ICellData> = {};
  if (sheet.cellData) {
    Object.entries(sheet.cellData).forEach(([cellRef, cellValue]) => {
      const value = cellValue as CellValue;
      cellData[cellRef] = {
        v: value.v,
        f: value.f,
      };
    });
  }

  // Convert abstract locale to Univer LocaleType
  const getUniverLocale = (locale?: string): LocaleType => {
    switch (locale) {
      case 'en-US':
      case 'EN_US':
        return LocaleType.EN_US;
      case 'zh-CN':
      case 'ZH_CN':
        return LocaleType.ZH_CN;
      default:
        return LocaleType.EN_US;
    }
  };

  return {
    id: data.id,
    name: data.name,
    appVersion: data.appVersion || '1.0.0',
    locale: getUniverLocale(data.locale),
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    styles: (data.styles as any) || {},
    sheets: {
      [sheetId]: {
        id: sheetId,
        name: sheet.name,
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        cellData: cellData as any, // Type assertion needed due to complex Univer types
        rowCount: sheet.rowCount || 1000,
        columnCount: sheet.columnCount || 26,
      }
    },
    sheetOrder: data.sheetOrder || [sheetId],
  };
};

export const UniverAdapter = forwardRef<SpreadsheetRef, SpreadsheetProps>(
  ({ initialData, onCellChange, onFormulaIntercept, onSelectionChange }, ref) => {
    const univerRef = useRef<OriginalUniverRef>(null);

    useImperativeHandle(ref, () => ({
      updateCell: (cellRef: string, value: CellValue) => {
        univerRef.current?.updateCell(cellRef, value);
      },
      getCellValue: (cellRef: string): string | number | null => {
        return univerRef.current?.getCellValue(cellRef) || null;
      },
      getRange: async (rangeRef: string): Promise<(string | number)[][]> => {
        return await univerRef.current?.getRange(rangeRef) || [];
      }
    }));

    // Wrap callbacks to handle type conversion
    const handleCellChange = (cellRef: string, univerCellData: ICellData) => {
      const abstractCellData: CellValue = {
        v: typeof univerCellData.v === 'boolean' ? undefined : univerCellData.v || undefined,
        f: univerCellData.f || undefined,
      };
      onCellChange(cellRef, abstractCellData);
    };

    // Memoize the converted data to prevent unnecessary re-initialization
    const univerData = useMemo(() => convertToUniverData(initialData), [initialData]);

    return (
      <UniverSpreadsheet
        ref={univerRef}
        initialData={univerData}
        onCellChange={handleCellChange}
        onFormulaIntercept={onFormulaIntercept}
        onSelectionChange={onSelectionChange}
      />
    );
  }
);