// dataConversion.ts - Data conversion utilities for Facade API
import { IWorkbookData, ICellData, LocaleType, Nullable, IStyleData, IObjectMatrixPrimitiveType } from '@univerjs/core';
import { WorkbookData, CellValue } from '@/components/spreadsheet/SpreadsheetInterface';
import { parseCellRef } from '../index';
import { ERROR_MESSAGES } from './constants';

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
  // Support both abstract format (style) and direct Univer format (s)
  const styleData = cellValue.style ?? (cellValue as unknown as { s?: unknown }).s;
  if (styleData !== undefined && isValidCellStyle(styleData)) {
    univerCell.s = styleData;
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
 * Convert cell data from Record<string, CellValue> to nested IObjectMatrixPrimitiveType<ICellData>
 * Structure: { [row: number]: { [col: number]: ICellData } }
 */
function convertCellDataToMatrix(cellDataRecord: Record<string, unknown>): IObjectMatrixPrimitiveType<ICellData> {
  const cellData: IObjectMatrixPrimitiveType<ICellData> = {};
  
  Object.entries(cellDataRecord).forEach(([cellRef, cellValue]) => {
    const indices = parseCellRef(cellRef);
    if (indices) {
      const { row, col } = indices;
      // Initialize row object if it doesn't exist
      cellData[row] ??= {};
      // Set the cell data at the specific row and column
      cellData[row][col] = convertCellValueToICellData(cellValue as CellValue);
    } else {
      console.warn(`Invalid cell reference: ${cellRef}`);
    }
  });
  
  return cellData;
}

/**
 * Convert abstract WorkbookData to Univer IWorkbookData
 */
export function convertToUniverData(data: WorkbookData): IWorkbookData {
  const sheetId = Object.keys(data.sheets)[0] ?? 'sheet-01';
  const sheet = data.sheets[sheetId];

  if (!sheet) {
    throw new Error(ERROR_MESSAGES.SHEET_ID_NOT_FOUND(sheetId));
  }

  const cellData = sheet.cellData ? convertCellDataToMatrix(sheet.cellData) : {};

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
 * Convert abstract WorkbookData to Univer IWorkbookData with support for multiple sheets
 */
export function convertToUniverDataMultiSheet(data: WorkbookData): IWorkbookData {
  // Convert styles with proper type checking
  const styles: Record<string, Nullable<IStyleData>> = {};
  if (data.styles && typeof data.styles === 'object') {
    Object.entries(data.styles).forEach(([key, value]) => {
      if (value !== null && value !== undefined) {
        styles[key] = value as Nullable<IStyleData>;
      }
    });
  }

  // Convert all sheets
  const sheets: Record<string, unknown> = {};
  const sheetOrder: string[] = [];

  Object.entries(data.sheets).forEach(([sheetId, sheet]) => {
    const cellData = sheet.cellData ? convertCellDataToMatrix(sheet.cellData) : {};
    
    // Create base sheet object with converted cell data
    const sheetObj: Record<string, unknown> = {
      id: sheetId,
      name: sheet.name,
      cellData,
      rowCount: sheet.rowCount ?? 1000,
      columnCount: sheet.columnCount ?? 26,
    };
    
    // Copy over additional properties from sheet (like mergeData, etc.)
    // This allows sheet-specific data to be preserved through conversion
    Object.entries(sheet).forEach(([key, value]) => {
      // Skip properties we've already handled
      if (!['id', 'name', 'cellData', 'rowCount', 'columnCount'].includes(key)) {
        sheetObj[key] = value;
      }
    });
    
    sheets[sheetId] = sheetObj;
    sheetOrder.push(sheetId);
  });

  const result: Record<string, unknown> = {
    id: data.id,
    name: data.name,
    appVersion: data.appVersion ?? '1.0.0',
    locale: getUniverLocale(data.locale),
    styles,
    sheets,
    sheetOrder,
  };
  
  // Copy over any additional workbook-level properties
  Object.entries(data).forEach(([key, value]) => {
    if (!['id', 'name', 'appVersion', 'locale', 'styles', 'sheets', 'sheetOrder'].includes(key)) {
      result[key] = value;
    }
  });

  return result as unknown as IWorkbookData;
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
 * Convert simple 2D array of values to CellValue[][] format
 * Used for importing simple data formats (CSV, TSV, etc.)
 */
export function convertSimpleArrayToCellValues(data: unknown[][]): CellValue[][] {
  return data.map(row =>
    row.map(cell => {
      if (cell === null || cell === undefined) {
        return { v: null };
      }
      if (typeof cell === 'string' || typeof cell === 'number' || typeof cell === 'boolean') {
        return { v: cell };
      }
      return { v: null };
    })
  );
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