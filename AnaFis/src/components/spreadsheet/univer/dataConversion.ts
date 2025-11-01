// dataConversion.ts - Data conversion utilities for Facade API
import { IWorkbookData, ICellData, LocaleType, Nullable, IStyleData, IObjectMatrixPrimitiveType } from '@univerjs/core';
import { WorkbookData, CellValue } from '../SpreadsheetInterface';
import { cellRefToIndices } from './univerUtils';

/**
 * Type guard to check if a value is a valid cell style
 */
function isValidCellStyle(value: unknown): value is NonNullable<ICellData['s']> {
  return value !== null && value !== undefined && typeof value === 'object';
}

/**
 * Type guard to check if a value is a valid cell type
 */
function isValidCellType(value: unknown): value is NonNullable<ICellData['t']> {
  return typeof value === 'number' || typeof value === 'string';
}

/**
 * Type guard to check if a value is a valid formula ID
 */
function isValidFormulaId(value: unknown): value is NonNullable<ICellData['si']> {
  return typeof value === 'string' || typeof value === 'number';
}

/**
 * Safely convert CellValue to ICellData with proper type checking
 */
function convertCellValueToICellData(cellValue: CellValue): ICellData {
  const univerCell: ICellData = {};

  // Handle basic value and formula
  if (cellValue.v !== undefined && cellValue.v !== null) {
    if (typeof cellValue.v === 'boolean') {
      univerCell.v = cellValue.v;
    } else if (typeof cellValue.v === 'string' || typeof cellValue.v === 'number') {
      univerCell.v = cellValue.v;
    }
  }

  if (cellValue.f) {
    univerCell.f = cellValue.f;
  }

  // Handle style with type checking
  if (cellValue.style !== undefined && isValidCellStyle(cellValue.style)) {
    univerCell.s = cellValue.style;
  }

  // Handle meta fields with proper type guards
  if (cellValue.meta) {
    // Custom data (maps to p) - cast to expected Univer type
    if (cellValue.meta.custom !== undefined && cellValue.meta.custom !== null) {
      univerCell.p = cellValue.meta.custom as NonNullable<ICellData['p']>;
    }

    // Cell type (maps to t)
    if (cellValue.meta.cellType !== undefined && isValidCellType(cellValue.meta.cellType)) {
      univerCell.t = cellValue.meta.cellType;
    }

    // Formula ID (maps to si)
    if (cellValue.meta.formulaId !== undefined && isValidFormulaId(cellValue.meta.formulaId)) {
      univerCell.si = cellValue.meta.formulaId;
    }

    // Custom fields (maps to custom) - cast to expected Univer type
    if (cellValue.meta.customFields !== undefined && cellValue.meta.customFields !== null) {
      univerCell.custom = cellValue.meta.customFields as NonNullable<ICellData['custom']>;
    }
  }

  return univerCell;
}

/**
 * Convert abstract CellValue to Univer ICellData
 */
export function convertToUniverCellValue(cellValue: CellValue): ICellData {
  return convertCellValueToICellData(cellValue);
}

/**
 * Convert abstract WorkbookData to Univer IWorkbookData
 */
export function convertToUniverData(data: WorkbookData): IWorkbookData {
  const sheetId = Object.keys(data.sheets)[0] ?? 'sheet-01';
  const sheet = data.sheets[sheetId];

  if (!sheet) {
    throw new Error(`Sheet with ID ${sheetId} not found`);
  }

  // Convert cellData from Record<string, CellValue> to nested IObjectMatrixPrimitiveType<ICellData>
  // Structure: { [row: number]: { [col: number]: ICellData } }
  const cellData: IObjectMatrixPrimitiveType<ICellData> = {};
  if (sheet.cellData) {
    Object.entries(sheet.cellData).forEach(([cellRef, cellValue]) => {
      const indices = cellRefToIndices(cellRef);
      if (indices) {
        const { row, col } = indices;
        // Initialize row object if it doesn't exist
        cellData[row] ??= {};
        // Set the cell data at the specific row and column
        cellData[row][col] = convertCellValueToICellData(cellValue);
      } else {
        console.warn(`Invalid cell reference: ${cellRef}`);
      }
    });
  }

  // Convert styles with proper type checking
  const styles: Record<string, Nullable<IStyleData>> = {};
  if (data.styles && typeof data.styles === 'object') {
    Object.entries(data.styles).forEach(([key, value]) => {
      if (value !== null && value !== undefined) {
        styles[key] = value as Nullable<IStyleData>;
      }
    });
  }

  return {
    id: data.id,
    name: data.name,
    appVersion: data.appVersion ?? '1.0.0',
    locale: getUniverLocale(data.locale),
    styles,
    sheets: {
      [sheetId]: {
        id: sheetId,
        name: sheet.name,
        cellData,
        rowCount: sheet.rowCount ?? 1000,
        columnCount: sheet.columnCount ?? 26,
      }
    },
    sheetOrder: data.sheetOrder ?? [sheetId],
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
export function convertFromUniverCellData(cellData: ICellData): CellValue {
  // Handle the value conversion with proper type checking
  let convertedValue: string | number | boolean | null = null;
  if (cellData.v !== undefined) {
    if (typeof cellData.v === 'boolean') {
      convertedValue = cellData.v;
    } else if (typeof cellData.v === 'string' || typeof cellData.v === 'number') {
      convertedValue = cellData.v;
    }
  }

  const result: CellValue = {
    v: convertedValue
  };

  // Only add properties if they have actual values (not undefined)
  if (cellData.f) {
    result.f = cellData.f;
  }

  if (cellData.s) {
    result.style = cellData.s;
  }

  // Handle meta fields - build meta object incrementally
  const meta: CellValue['meta'] = {};

  // Handle cell type
  if (cellData.t !== undefined) {
    meta.cellType = cellData.t;
  }

  // Handle formula ID
  if (cellData.si !== undefined && cellData.si !== null) {
    meta.formulaId = cellData.si;
  }

  // Handle custom fields - prioritize cellData.custom over cellData.p
  if (cellData.custom !== undefined) {
    meta.customFields = cellData.custom;
  } else if (cellData.p) {
    meta.custom = cellData.p;
  }

  // Only set meta if it has properties
  if (Object.keys(meta).length > 0) {
    result.meta = meta;
  }

  return result;
}