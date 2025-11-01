// univer/index.ts - Optimized exports for better tree shaking and performance

// Primary components (most commonly used)
export { UniverAdapter } from './UniverAdapter';
export { default as UniverSpreadsheet } from './UniverSpreadsheet';

// Core utilities (frequently used)
export {
  columnToLetter,
  letterToColumn,
  parseRange,
  rangeToA1,
  cellRefToIndices,
  clearAllCaches,
  startPeriodicCacheCleanup,
  stopPeriodicCacheCleanup
} from './univerUtils';

// Alias for backward compatibility
export { clearAllCaches as clearUtilCaches } from './univerUtils';

// Error handling (essential)
export {
  UniverError,
  UniverValidationError,
  UniverOperationError,
  safeUniverOperation,
  isValidCellRef,
  isValidRangeRef,
  normalizeCellRef,
  normalizeRangeRef,
  clearValidationCache
} from './errors';

// Spreadsheet operations (core functionality)
export {
  getWorkbook,
  updateCell,
  getCellValue,
  getRange,
  getRangeFull,
  getAllSheetsData,
  getSelection
} from './spreadsheetOperations';

// Re-export data conversion utilities explicitly for tree shaking
export {
  convertToUniverCellValue,
  convertToUniverData,
  getUniverLocale,
  convertFromUniverCellData
} from './dataConversion';

// Re-export custom formulas explicitly for tree shaking
export {
  registerCustomFunctions,
  CUSTOM_FUNCTION_NAMES
} from './customFormulas';

// Re-export table format extraction explicitly for tree shaking
export type {
  CellFormatInfo,
  BorderInfo,
  ExtractionOptions,
  FormattedTable
} from './tableFormatExtraction';
export {
  TableDataTransformer,
  extractFormattedTable,
  createTableTransformer
} from './tableFormatExtraction';
// Explicitly re-export from exportService to avoid conflicts
export {
  ExportService,
  type ExportResult,
  type ExportFormat,
  type ExportRangeMode,
  getFileExtension,
  getFilterName
} from './exportService';
// Re-export other exportIntegration items without conflicts
export {
  type ExportConfig,
  type ExportOptions
} from '../../../types/export';

// Facade is commonly used, so export directly
export {
  SpreadsheetFacade,
  createSpreadsheetFacade,
  getSpreadsheetFacade,
  initializeFacade,
  resetFacade,
  type ISpreadsheetFacade
} from './facade';

// Alias for backward compatibility
export {
  SpreadsheetFacade as UniverFacade,
  createSpreadsheetFacade as createUniverFacade,
  getSpreadsheetFacade as getUniverFacade,
  initializeFacade as initializeUniverFacade,
  resetFacade as resetUniverFacade,
  type ISpreadsheetFacade as IUniverFacade
} from './facade';

// Error boundaries for better error isolation
export { LightweightErrorBoundary, UniverErrorBoundary, useErrorHandler } from './UniverErrorBoundary';