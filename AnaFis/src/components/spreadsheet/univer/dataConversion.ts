// dataConversion.ts - Functions for converting between abstract and Univer data formats
import { IWorkbookData, ICellData, LocaleType } from '@univerjs/core';
import { WorkbookData, CellValue } from '../SpreadsheetInterface';

/**
 * Convert abstract WorkbookData to Univer IWorkbookData
 */
export function convertToUniverData(data: WorkbookData): IWorkbookData {
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
        s: value.style as any,
        p: value.meta?.custom as any,
        t: value.meta?.cellType as any,
        si: value.meta?.formulaId as any,
        custom: value.meta?.customFields as any,
      };
    });
  }

  return {
    id: data.id,
    name: data.name,
    appVersion: data.appVersion || '1.0.0',
    locale: getUniverLocale(data.locale),
    styles: (data.styles as any) || {},
    sheets: {
      [sheetId]: {
        id: sheetId,
        name: sheet.name,
        cellData: cellData as any, // Type assertion needed due to complex Univer types
        rowCount: sheet.rowCount || 1000,
        columnCount: sheet.columnCount || 26,
      }
    },
    sheetOrder: data.sheetOrder || [sheetId],
  };
}

/**
 * Convert abstract locale to Univer LocaleType
 */
export function getUniverLocale(locale?: string): LocaleType {
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
}

/**
 * Convert Univer ICellData to abstract CellValue
 */
export function convertFromUniverCell(univerCellData: ICellData): CellValue {
  return {
    v: typeof univerCellData.v === 'boolean' ? undefined : univerCellData.v || undefined,
    f: univerCellData.f || undefined,
    style: univerCellData.s,
    meta: univerCellData.p ? { custom: univerCellData.p } : undefined
  };
}

/**
 * Convert abstract CellValue to Univer format for updates
 */
export function convertToUniverCellValue(value: CellValue): { v?: string | number; f?: string } {
  return {
    v: value.v === null || value.v === undefined ? undefined :
       typeof value.v === 'boolean' ? (value.v ? 'TRUE' : 'FALSE') : value.v,
    f: value.f
  };
}