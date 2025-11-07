import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { SpreadsheetRef, CellValue } from '@/tabs/spreadsheet/types/SpreadsheetInterface';
import { getInputType, parseRange, getRangeFormat, rangesOverlap } from '@/tabs/spreadsheet/components/sidebar/utils/rangeUtils';
import { extractStartCell } from '@/tabs/spreadsheet/utils/rangeUtils';

export interface ConversionResult {
  value: number;
  formatted_result: string;
  conversion_factor: number;
}

export interface UnitConversionOptions {
  spreadsheetRef: React.RefObject<SpreadsheetRef | null> | undefined;
  fromUnit: string;
  toUnit: string;
  value: string;
}

/**
 * Custom hook for handling unit conversion operations on spreadsheet
 */
export const useSpreadsheetUnitConversion = ({ spreadsheetRef, fromUnit, toUnit, value }: UnitConversionOptions) => {
  const [result, setResult] = useState<string>('');
  const [error, setError] = useState<string>('');
  const [isConverting, setIsConverting] = useState<boolean>(false);
  const [compatibilityError, setCompatibilityError] = useState<string>('');

  // Check unit compatibility when both units are selected
  const checkCompatibility = useCallback(async () => {
    try {
      const compatible: boolean = await invoke('check_unit_compatibility', {
        fromUnit,
        toUnit
      });
      if (!compatible) {
        setCompatibilityError(`Units ${fromUnit} and ${toUnit} are not compatible`);
      } else {
        setCompatibilityError('');
      }
    } catch (err) {
      setCompatibilityError(`Error checking compatibility: ${String(err)}`);
    }
  }, [fromUnit, toUnit]);

  // Display result in sidebar
  const handleSidebarDisplay = useCallback(async (input: string, inputType: string) => {
    let numValue: number;

    if (inputType === 'number') {
      numValue = parseFloat(input);
    } else { // inputType === 'cell'
      if (!spreadsheetRef?.current) {
        throw new Error('Spreadsheet not available');
      }
      const cellValue = await spreadsheetRef.current.getCellValue(input);
      if (cellValue === null) {
        throw new Error(`Cell ${input} is empty`);
      }
      numValue = typeof cellValue === 'number' ? cellValue : parseFloat(String(cellValue));
      if (isNaN(numValue)) {
        throw new Error(`Cell ${input} does not contain a valid number`);
      }
    }

    const conversionResult: ConversionResult = await invoke('convert_value', {
      request: { value: numValue, from_unit: fromUnit, to_unit: toUnit }
    });

    // Show conversion rate if input was empty (defaulted to 1)
    const isConversionRate = value.trim() === '';
    const prefix = isConversionRate ? `Conversion rate: 1 ${fromUnit} = ` : '';
    setResult(`${prefix}${conversionResult.value.toFixed(6)} ${toUnit}`);
  }, [fromUnit, toUnit, value, spreadsheetRef]);

  // Replace value in-place
  const handleInPlaceReplacement = useCallback(async (cellRef: string) => {
    if (!spreadsheetRef?.current) {
      throw new Error('Spreadsheet not available');
    }
    const cellValue = await spreadsheetRef.current.getCellValue(cellRef);

    // Convert calculated result (works for both numbers and formulas)
    const numValue = typeof cellValue === 'number' ? cellValue : parseFloat(String(cellValue));
    if (isNaN(numValue)) {
      throw new Error(`Cell ${cellRef} does not contain a valid number`);
    }

    // Get conversion factor for potential formula modification
    const conversionResult: ConversionResult = await invoke('convert_value', {
      request: { value: 1, from_unit: fromUnit, to_unit: toUnit }
    });
    const conversionFactor = conversionResult.value;

    // Apply conversion factor to the current value
    const convertedValue = numValue * conversionFactor;

    // OPTIMIZED: Direct update instead of clear + set (removes double update)
    await spreadsheetRef.current.updateCell(cellRef, { v: convertedValue });

    setResult(`Replaced ${cellRef}: ${numValue} ${fromUnit} → ${convertedValue.toFixed(6)} ${toUnit} (factor: ${conversionFactor.toFixed(6)})`);
  }, [fromUnit, toUnit, spreadsheetRef]);

  // Fill output cells with converted value
  const handleFillOutput = useCallback(async (fromInput: string, toInput: string, fromType: string, toType: string) => {
    // Get the value to convert
    let numValue: number;
    if (fromType === 'number') {
      numValue = parseFloat(fromInput);
    } else { // fromType === 'cell'
      if (!spreadsheetRef?.current) {
        throw new Error('Spreadsheet not available');
      }
      const cellValue = await spreadsheetRef.current.getCellValue(fromInput);
      if (cellValue === null) {
        throw new Error(`Cell ${fromInput} is empty`);
      }
      numValue = typeof cellValue === 'number' ? cellValue : parseFloat(String(cellValue));
      if (isNaN(numValue)) {
        throw new Error(`Cell ${fromInput} does not contain a valid number`);
      }
    }

    // Convert the value
    const conversionResult: ConversionResult = await invoke('convert_value', {
      request: { value: numValue, from_unit: fromUnit, to_unit: toUnit }
    });

    // Fill output cells
    if (!spreadsheetRef?.current) {
      throw new Error('Spreadsheet not available');
    }

    if (toType === 'cell') {
      // Single cell output
      await spreadsheetRef.current.updateCell(toInput, { v: conversionResult.value });
    } else { // toType === 'range'
      // Range output - use efficient range update instead of individual cells
      const rangeFormat = getRangeFormat(toInput);
      // Create 2D array filled with the converted value
      const values: CellValue[][] = Array.from({ length: rangeFormat.rows }, () =>
        Array.from({ length: rangeFormat.cols }, () => ({ v: conversionResult.value }))
      );
      // Extract starting cell from range (e.g., "R10:R15" → "R10")
      const startCell = extractStartCell(toInput);
      await spreadsheetRef.current.updateRange(startCell, values);
    }

    setResult(`Filled ${toType === 'cell' ? '1 cell' : `${getRangeFormat(toInput).rows * getRangeFormat(toInput).cols} cells`} with ${conversionResult.value.toFixed(6)} ${toUnit}`);
  }, [fromUnit, toUnit, spreadsheetRef]);

  // Apply conversion factor to entire range in-place
  const handleSameRangeConversion = useCallback(async (range: string) => {
    // Get conversion factor
    const conversionResult: ConversionResult = await invoke('convert_value', {
      request: { value: 1, from_unit: fromUnit, to_unit: toUnit }
    });
    const conversionFactor = conversionResult.value;

    if (!spreadsheetRef?.current) {
      throw new Error('Spreadsheet reference not available');
    }

    // OPTIMIZED: Read entire range at once
    const rangeData = await spreadsheetRef.current.getRange(range);

    // Process and convert all values
    const convertedData = rangeData.map(row =>
      row.map(cellValue => {
        if (cellValue === '') {
          return { v: cellValue }; // Keep empty cells as-is
        }

        const numValue = typeof cellValue === 'number' ? cellValue : parseFloat(String(cellValue));
        if (isNaN(numValue)) {
          return { v: cellValue }; // Keep non-numeric cells as-is
        }

        return { v: numValue * conversionFactor };
      })
    );

    // Extract starting cell from range (e.g., "R10:R15" → "R10")
    // updateRange expects only the starting cell, not the full range
    const startCell = extractStartCell(range);

    // OPTIMIZED: Single range update instead of individual cell updates
    await spreadsheetRef.current.updateRange(startCell, convertedData);

    const totalCells = rangeData.flat().length;
    const convertedCount = convertedData.flat().filter(cell =>
      typeof cell.v === 'number' && !isNaN(cell.v)
    ).length;

    setResult(`Applied conversion factor ${conversionFactor.toFixed(6)} to ${convertedCount}/${totalCells} cells in ${range}`);
  }, [fromUnit, toUnit, spreadsheetRef]);

  // Convert range to range
  const handleRangeToRange = useCallback(async (fromRange: string, toRange: string) => {
    const fromCells = parseRange(fromRange);
    const toCells = parseRange(toRange);

    // Check for overlapping ranges (not same range)
    if (fromRange !== toRange && rangesOverlap(fromRange, toRange)) {
      throw new Error('Overlapping ranges not supported. Use same range for in-place conversion.');
    }

    // Check if ranges have same format (dimensions)
    const fromFormat = getRangeFormat(fromRange);
    const toFormat = getRangeFormat(toRange);

    if (fromFormat.rows !== toFormat.rows || fromFormat.cols !== toFormat.cols) {
      throw new Error(`Range formats must match: ${fromFormat.rows}x${fromFormat.cols} vs ${toFormat.rows}x${toFormat.cols}`);
    }

    // Special case: Same range → Apply conversion factor to formulas
    if (fromRange === toRange) {
      await handleSameRangeConversion(fromRange);
      return;
    }

    // OPTIMIZED: Read entire range at once instead of individual cells
    if (!spreadsheetRef?.current) {
      throw new Error('Spreadsheet not available');
    }
    const rangeData = await spreadsheetRef.current.getRange(fromRange);

    const values: number[] = [];
    const cellMapping: Array<{ fromCell: string; toCell: string; value: number; isFormula: boolean }> = [];

    // Process the 2D range data
    const flatData = rangeData.flat();
    const minLength = Math.min(flatData.length, fromCells.length, toCells.length);

    for (let i = 0; i < minLength; i++) {
      const fromCell = fromCells[i];
      const toCell = toCells[i];
      const cellValue = flatData[i];

      if (fromCell === undefined || toCell === undefined) {
        continue; // Skip if cells are undefined
      }

      if (cellValue === '') {
        continue; // Skip empty cells
      }

      // Convert calculated result (works for both numbers and formulas)
      const numValue = typeof cellValue === 'number' ? cellValue : parseFloat(String(cellValue));
      if (isNaN(numValue)) {
        continue; // Skip non-numeric cells
      }

      values.push(numValue);
      cellMapping.push({
        fromCell,
        toCell,
        value: numValue,
        isFormula: false // We convert the result, not the formula
      });
    }

    if (values.length === 0) {
      throw new Error('No valid numeric values found in source range');
    }

    // Convert all values
    const conversionResults = await Promise.all(
      values.map(value => invoke<ConversionResult>('convert_value', {
        request: { value, from_unit: fromUnit, to_unit: toUnit }
      }))
    );

    // Update output cells using range operation when possible
    // Check if we can use a single range update (same range dimensions)
    const targetFormat = getRangeFormat(toRange);
    if (cellMapping.length === targetFormat.rows * targetFormat.cols) {
      // Create 2D array for range update
      const rangeValues: CellValue[][] = Array.from({ length: targetFormat.rows }, () =>
        Array.from({ length: targetFormat.cols }, () => ({ v: null }))
      );

      // Fill the 2D array with converted values
      for (let i = 0; i < cellMapping.length; i++) {
        const mapping = cellMapping[i];
        if (mapping === undefined) {
          continue;
        }

        const conversionResult = conversionResults[i];
        if (conversionResult !== undefined) {
          const row = Math.floor(i / targetFormat.cols);
          const col = i % targetFormat.cols;
          const rangeRow = rangeValues[row];
          if (rangeRow !== undefined) {
            rangeRow[col] = { v: conversionResult.value };
          }
        }
      }

      // Single range update instead of multiple cell updates
      // Extract starting cell from toRange (e.g., "R10:R15" → "R10")
      const startCell = extractStartCell(toRange);
      await spreadsheetRef.current.updateRange(startCell, rangeValues);
    } else {
      // Fallback to individual cell updates for irregular mappings
      for (let i = 0; i < cellMapping.length; i++) {
        const mapping = cellMapping[i];
        if (mapping === undefined) {
          continue;
        }

        const conversionResult = conversionResults[i];
        if (conversionResult !== undefined) {
          await spreadsheetRef.current.updateCell(mapping.toCell, { v: conversionResult.value });
        }
      }
    }

    setResult(`Converted ${values.length} values from ${fromRange} to ${toRange}`);
  }, [fromUnit, toUnit, spreadsheetRef, handleSameRangeConversion]);

  // Main conversion logic dispatcher
  const applyConversionLogic = useCallback(async (fromInput: string, toInput: string, fromType: string, toType: string) => {
    // Rule 1: Number/Cell + Empty → Display result in sidebar
    if ((fromType === 'number' || fromType === 'cell') && toType === 'empty') {
      await handleSidebarDisplay(fromInput, fromType);
      return;
    }

    // Rule 2: Range + Empty → Error
    if (fromType === 'range' && toType === 'empty') {
      throw new Error('Range input requires output destination');
    }

    // Rule 3: Number/Cell + Different Cell/Range → Fill all "To" cells
    if ((fromType === 'number' || fromType === 'cell') && (toType === 'cell' || toType === 'range')) {
      await handleFillOutput(fromInput, toInput, fromType, toType);
      return;
    }

    // Rule 4: Range + Same-size Range → Convert each cell
    if (fromType === 'range' && toType === 'range') {
      await handleRangeToRange(fromInput, toInput);
      return;
    }

    // Rule 5: Same Cell → Replace in-place
    if (fromType === 'cell' && toType === 'cell' && fromInput === toInput) {
      await handleInPlaceReplacement(fromInput);
      return;
    }

    throw new Error(`Unsupported conversion: ${fromType} → ${toType}`);
  }, [handleSidebarDisplay, handleFillOutput, handleRangeToRange, handleInPlaceReplacement]);

  // Main convert function
  const convert = useCallback(async (outputTarget: string = '') => {
    setError('');
    setResult('');

    if (!fromUnit || !toUnit) {
      setError('Please enter both units');
      return;
    }

    // Check if spreadsheet implementation is ready for cell operations
    if (spreadsheetRef?.current && !spreadsheetRef.current.isReady()) {
      setError('Spreadsheet is not ready for cell operations. Please wait a moment and try again.');
      return;
    }

    setIsConverting(true);
    try {
      // Parse input and output
      const fromInput = value.trim() || '1';
      const toInput = outputTarget.trim();

      const fromType = getInputType(fromInput);
      const toType = toInput ? getInputType(toInput) : 'empty';

      // Apply conversion logic based on input/output types
      await applyConversionLogic(fromInput, toInput, fromType, toType);

    } catch (err) {
      setError(String(err));
    } finally {
      setIsConverting(false);
    }
  }, [value, fromUnit, toUnit, spreadsheetRef, applyConversionLogic]);

  return {
    result,
    error,
    isConverting,
    compatibilityError,
    checkCompatibility,
    convert,
    clearResult: () => setResult(''),
    clearError: () => setError('')
  };
};