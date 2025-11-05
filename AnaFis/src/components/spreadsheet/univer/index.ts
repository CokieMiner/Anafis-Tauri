// univer/index.ts - Organized exports with clear structure

// ============================================================================
// CORE COMPONENTS
// ============================================================================
export { UniverAdapter } from './core/UniverAdapter';
export { default as UniverSpreadsheet } from './core/UniverSpreadsheet';
export { UniverErrorBoundary } from './core/UniverErrorBoundary';

// ============================================================================
// OPERATIONS
// ============================================================================

// Import/Export Services
export {
  ExportService,
  type ExportResult,
  type ExportFormat,
  getFileExtension,
  getFilterName
} from './operations/exportService';

export {
  ImportService,
  type ImportFormat,
  type ImportResult,
  type FileMetadata
} from './operations/importService';

// Bulk Import Operations
export {
  bulkLoadSheetData,
  bulkLoadSheetDataFromMatrix,
  bulkLoadSheetDataWithOffset,
  clearSheetData,
  type ImportCellData,
  type ImportSheetData,
  type BulkImportOptions
} from './operations/bulkImportOperations';

// Facade Operations (Univer API wrappers)
export {
  getWorkbook,
  updateCell,
  getCellValue,
  getRange,
  getRangeFull,
  getSelection,
  type UniverRef
} from './operations/facadeOperations';

// Uncertainty Operations (High-level uncertainty propagation)
export {
  validateUncertaintySetup,
  runUncertaintyPropagation,
  type ValidationResult,
  type PropagationResult,
  type Variable
} from './operations/uncertaintyOperations';

// ============================================================================
// UTILITIES
// ============================================================================

// A1 Notation & Column Conversion
export {
  parseRange,
  rangeToA1,
  determineUsedRange,
  rangesIntersect,
  type RangeBounds
} from './utils/univerUtils';

// Range Utilities (re-exported from generic utils for convenience)
export {
  extractStartCell,
  extractEndCell,
  isSingleCell,
  boundsToA1StartCell,
  boundsToA1Range,
  getRangeRowCount,
  getRangeColumnCount
} from './utils/rangeUtils';

// Cell reference utilities
export {
  columnToLetter,
  letterToColumn,
  parseCellRef,
  parseRangeOrCell
} from './utils/cellUtils';

// Data Conversion (Abstract ↔ Univer formats)
export {
  convertToUniverCellValue,
  convertToUniverData,
  convertToUniverDataMultiSheet,
  getUniverLocale,
  convertFromUniverCellData,
  convertSimpleArrayToCellValues
} from './utils/dataConversion';

// Validation
export {
  isValidCellRef,
  isValidRangeRef,
  normalizeCellRef,
  normalizeRangeRef,
  clearValidationCache
} from './utils/validation';

// Performance Monitoring
export {
  withPerformanceMonitoring
} from './utils/performance';

// ============================================================================
// ERROR HANDLING
// ============================================================================
export {
  SpreadsheetError,
  SpreadsheetValidationError,
  SpreadsheetOperationError,
  handleSpreadsheetError,
  safeSpreadsheetOperation
} from './utils/errors';

// ============================================================================
// FORMULAS
// ============================================================================
export {
  registerCustomFunctions,
  CUSTOM_FUNCTION_NAMES
} from './formulas/customFormulas';

// ============================================================================
// EXTERNAL TYPE RE-EXPORTS
// ============================================================================
export {
  type ExportOptions
} from '@/types/export';

export {
  ErrorCode,
  type ErrorResponse,
  type CommandResult,
  isErrorResponse,
  getErrorMessage,
  getDetailedErrorMessage
} from '@/types/error';
