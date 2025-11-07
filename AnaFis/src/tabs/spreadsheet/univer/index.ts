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
  type FileMetadata
} from './operations/importService';

// Bulk Import Operations
export {
  bulkLoadSheetDataFromMatrix,
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
  type PropagationResult,
  type Variable
} from './operations/uncertaintyOperations';

// ============================================================================
// UTILITIES
// ============================================================================

// A1 Notation & Column Conversion
export {
  rangeToA1,
  determineUsedRange,
  rangesIntersect
} from './utils/univerUtils';


// Cell reference utilities
export {
  columnToLetter,
  letterToColumn,
  parseCellRef,
  parseRange,
  parseRangeOrCell,
  type RangeBounds
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
  normalizeRangeRef
} from './utils/validation';

// Range Validation
export { RangeValidator } from './utils/RangeValidator';

// Error Formatting
export { formatSpreadsheetError } from './utils/errors';

// ============================================================================
// ERROR HANDLING
// ============================================================================
export {
  SpreadsheetError,
  SpreadsheetValidationError,
  SpreadsheetOperationError,
  safeSpreadsheetOperation
} from './utils/errors';

// ============================================================================
// FORMULAS
// ============================================================================
export {
  registerCustomFunctions
} from './formulas/customFormulas';

// ============================================================================
// EXTERNAL TYPE RE-EXPORTS
// ============================================================================
export {
  type ExportOptions
} from '@/core/types/export';

export {
  ErrorCode,
  type ErrorResponse,
  type CommandResult,
  isErrorResponse,
  getErrorMessage,
  getDetailedErrorMessage
} from '@/core/types/error';
