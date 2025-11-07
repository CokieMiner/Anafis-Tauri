import { useCallback } from 'react';
import { parseRange } from '@/tabs/spreadsheet/univer/utils/cellUtils';
import type { ImportResult, FileMetadata, ImportTargetMode } from '@/core/types/import';
import type { DataSequence } from '@/core/types/dataLibrary';

/**
 * Custom hook for validating import ranges
 * Handles validation for both file imports and library imports
 */
export const useImportValidation = () => {
  /**
   * Validate range selection for file import
   */
  const validateFileRange = useCallback((
    targetRange: string,
    fileMetadata: FileMetadata | null,
    targetMode: ImportTargetMode
  ): ImportResult['rangeValidation'] | null => {
    if (!fileMetadata?.rowCount || !fileMetadata.columnCount || targetMode !== 'currentRange') {
      return null;
    }

    const warnings: string[] = [];
    let willTruncate = false;

    const rangeBounds = parseRange(targetRange);
    if (!rangeBounds) {
      return {
        isValid: false,
        warnings: ['Invalid range format'],
        willTruncate: false
      };
    }

    const selectedRows = rangeBounds.endRow - rangeBounds.startRow + 1;
    const selectedColumns = rangeBounds.endCol - rangeBounds.startCol + 1;

    // Check if range is too small
    if (1 < selectedRows && selectedRows < fileMetadata.rowCount || 1 < selectedColumns && selectedColumns < fileMetadata.columnCount) {
      willTruncate = true;
      warnings.push(`Selected range (${selectedRows}×${selectedColumns}) is smaller than file data (${fileMetadata.rowCount}×${fileMetadata.columnCount}). Data will be truncated to fit.`);
    }

    // Special case: single cell
    if (selectedRows === 1 && selectedColumns === 1) {
      warnings.push('Single cell selected. This will be treated as the top-left corner of the import area.');
    }

    return {
      isValid: true,
      warnings,
      willTruncate,
      selectedRange: {
        rows: selectedRows,
        columns: selectedColumns
      }
    };
  }, []);

  /**
   * Validate data range for library import
   */
  const validateLibraryDataRange = useCallback((
    libraryDataRange: string,
    sequence: DataSequence | undefined
  ): ImportResult['rangeValidation'] | null => {
    if (!sequence) {
      return null;
    }

    const sequenceRows = sequence.data.length;
    const sequenceColumns = 1; // Data is always 1 column

    const warnings: string[] = [];
    let willTruncate = false;

    const rangeBounds = parseRange(libraryDataRange);
    if (!rangeBounds) {
      return {
        isValid: false,
        warnings: ['Invalid range format'],
        willTruncate: false
      };
    }

    const selectedRows = rangeBounds.endRow - rangeBounds.startRow + 1;
    const selectedColumns = rangeBounds.endCol - rangeBounds.startCol + 1;

    if (selectedRows < sequenceRows || selectedColumns < sequenceColumns) {
      if (!(selectedRows === 1 && selectedColumns === 1)) {
        willTruncate = true;
        warnings.push(`Selected range (${selectedRows}×${selectedColumns}) is smaller than data (${sequenceRows}×1). Data will be truncated.`);
      }
    }

    if (selectedRows === 1 && selectedColumns === 1) {
      warnings.push('Single cell selected. Data will start here.');
    }

    return {
      isValid: true,
      warnings,
      willTruncate,
      selectedRange: { rows: selectedRows, columns: selectedColumns }
    };
  }, []);

  /**
   * Validate uncertainty range for library import
   */
  const validateLibraryUncertaintyRange = useCallback((
    libraryUncertaintyRange: string,
    sequence: DataSequence | undefined,
    includeUncertainties: boolean
  ): ImportResult['rangeValidation'] | null => {
    if (!sequence?.uncertainties || !includeUncertainties) {
      return null;
    }

    const sequenceRows = sequence.uncertainties.length;
    const sequenceColumns = 1; // Uncertainties are always 1 column

    const warnings: string[] = [];
    let willTruncate = false;

    const rangeBounds = parseRange(libraryUncertaintyRange);
    if (!rangeBounds) {
      return {
        isValid: false,
        warnings: ['Invalid range format'],
        willTruncate: false
      };
    }

    const selectedRows = rangeBounds.endRow - rangeBounds.startRow + 1;
    const selectedColumns = rangeBounds.endCol - rangeBounds.startCol + 1;

    if (selectedRows < sequenceRows || selectedColumns < sequenceColumns) {
      if (!(selectedRows === 1 && selectedColumns === 1)) {
        willTruncate = true;
        warnings.push(`Selected range (${selectedRows}×${selectedColumns}) is smaller than uncertainties (${sequenceRows}×1). Data will be truncated.`);
      }
    }

    if (selectedRows === 1 && selectedColumns === 1) {
      warnings.push('Single cell selected. Uncertainties will start here.');
    }

    return {
      isValid: true,
      warnings,
      willTruncate,
      selectedRange: { rows: selectedRows, columns: selectedColumns }
    };
  }, []);

  return {
    validateFileRange,
    validateLibraryDataRange,
    validateLibraryUncertaintyRange
  };
};
