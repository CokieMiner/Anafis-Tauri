// univer/index.ts - Organized exports with clear structure

export { UniverAdapter } from './core/UniverAdapter';
export { UniverErrorBoundary } from './core/UniverErrorBoundary';
export { registerCustomFunctions } from './formulas/customFormulas';
// Import/Export Services
export { ExportService } from './operations/exportService';
// Facade Operations (Univer API wrappers)
export {
  getCellValue,
  getRange,
  getRangeFull,
  getSelection,
  updateCell,
} from './operations/facadeOperations';
export { ImportService } from './operations/importService';
// Cell reference utilities
export {
  columnToLetter,
  letterToColumn,
  parseCellRef,
  parseRange,
} from './utils/cellUtils';
// Data Conversion (Abstract ↔ Univer formats)
export {
  convertFromUniverCellData,
  convertSimpleArrayToCellValues,
  convertToUniverCellValue,
  convertToUniverData,
  convertToUniverDataMultiSheet,
} from './utils/dataConversion';

export {
  SpreadsheetValidationError,
  safeSpreadsheetOperation,
  safeSpreadsheetOperationSync,
} from './utils/errors';
// A1 Notation & Column Conversion
export {
  determineUsedRange,
  rangeToA1,
} from './utils/univerUtils';
