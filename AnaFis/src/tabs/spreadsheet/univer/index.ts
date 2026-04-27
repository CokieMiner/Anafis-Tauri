// univer/index.ts - Organized exports with clear structure

export { UniverAdapter } from '@/tabs/spreadsheet/univer/core/UniverAdapter';
export { UniverErrorBoundary } from '@/tabs/spreadsheet/univer/core/UniverErrorBoundary';
export { registerCustomFunctions } from '@/tabs/spreadsheet/univer/formulas/customFormulas';

// Facade Operations (Univer API wrappers)
export {
  getCellValue,
  getRange,
  getRangeFull,
  getSelection,
  updateCell,
} from '@/tabs/spreadsheet/univer/operations/facadeOperations';

// Cell reference utilities
export {
  letterToColumn,
  parseCellRef,
  parseRange,
} from '@/tabs/spreadsheet/univer/utils/cellUtils';

// Data Conversion (Abstract ↔ Univer formats)
export {
  convertFromUniverCellData,
  convertSimpleArrayToCellValues,
  convertToUniverData,
} from '@/tabs/spreadsheet/univer/utils/dataConversion';

export {
  SpreadsheetValidationError,
  safeSpreadsheetOperation,
  safeSpreadsheetOperationSync,
} from '@/tabs/spreadsheet/univer/utils/errors';

// A1 Notation & Column Conversion
export {
  determineUsedRange,
  rangeToA1,
} from '@/tabs/spreadsheet/univer/utils/univerUtils';
