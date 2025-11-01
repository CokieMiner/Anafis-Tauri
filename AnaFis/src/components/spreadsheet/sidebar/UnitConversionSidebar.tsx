import React, { useState, useEffect, useCallback, useMemo } from 'react';
import {
  Box,
  Typography,
  IconButton,
  TextField,
  Button,
  Alert,
  Chip,
  List,
  ListItemButton,
  ListItemText,
  CircularProgress
} from '@mui/material';
import {
  Close as CloseIcon,
  SwapHoriz as SwapIcon,
  Search as SearchIcon,
  Transform as ConvertIcon
} from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';
import { SpreadsheetRef, CellValue } from '../SpreadsheetInterface';
import { useSpreadsheetSelection } from '@/hooks/useSpreadsheetSelection';
import { sidebarStyles } from '@/utils/sidebarStyles';
import SidebarCard from './SidebarCard';
import { anafisColors } from '@/themes';
import { spreadsheetEventBus } from '../SpreadsheetEventBus';

interface UnitConversionSidebarProps {
  open: boolean;
  onClose: () => void;
  univerRef?: React.RefObject<SpreadsheetRef | null>;
  onSelectionChange?: (selection: string) => void;
  // Lifted state for persistence
  category: string;
  setCategory: (category: string) => void;
  fromUnit: string;
  setFromUnit: (unit: string) => void;
  toUnit: string;
  setToUnit: (unit: string) => void;
  value: string;
  setValue: (value: string) => void;
}

interface ConversionResult {
  value: number;
  formatted_result: string;
  conversion_factor: number;
}

interface UnitInfo {
  symbol: string;
  name: string;
  category: string;
  description: string;
}

type FocusedInputType = 'value' | 'outputTarget' | null;

// Cache for unit metadata (shared across all instances)
const unitMetadataCache = new Map<string, UnitInfo>();
let cacheInitialized = false;

// Helper functions moved outside component to prevent recreation
const colToNum = (col: string): number => {
  let result = 0;
  for (let i = 0; i < col.length; i++) {
    result = result * 26 + (col.charCodeAt(i) - 65 + 1);
  }
  return result;
};

const numToCol = (num: number): string => {
  let colStr = '';
  let temp = num;
  while (temp > 0) {
    temp--;
    colStr = String.fromCharCode(65 + (temp % 26)) + colStr;
    temp = Math.floor(temp / 26);
  }
  return colStr;
};

// Cached regex patterns for better performance
const RANGE_REGEX = /^([A-Z]+)(\d+):([A-Z]+)(\d+)$/;
const CELL_REGEX = /^[A-Z]+\d+$/;

const parseRange = (rangeRef: string): string[] => {
  const match = rangeRef.match(RANGE_REGEX);
  if (!match?.[1] || !match[2] || !match[3] || !match[4]) {
    throw new Error(`Invalid range format: ${rangeRef}`);
  }

  const startCol = match[1];
  const startRow = parseInt(match[2]);
  const endCol = match[3];
  const endRow = parseInt(match[4]);

  const startColNum = colToNum(startCol);
  const endColNum = colToNum(endCol);

  const cells: string[] = [];
  for (let row = startRow; row <= endRow; row++) {
    for (let colNum = startColNum; colNum <= endColNum; colNum++) {
      const colStr = numToCol(colNum);
      cells.push(`${colStr}${row}`);
    }
  }
  return cells;
};

const getRangeFormat = (rangeRef: string): { rows: number; cols: number } => {
  const match = rangeRef.match(RANGE_REGEX);
  if (!match?.[1] || !match[2] || !match[3] || !match[4]) {
    throw new Error(`Invalid range format: ${rangeRef}`);
  }

  const startCol = match[1];
  const startRow = parseInt(match[2]);
  const endCol = match[3];
  const endRow = parseInt(match[4]);

  const rows = endRow - startRow + 1;
  const cols = colToNum(endCol) - colToNum(startCol) + 1;

  return { rows, cols };
};

const rangesOverlap = (range1: string, range2: string): boolean => {
  // Parse range1
  const match1 = range1.match(RANGE_REGEX);
  if (!match1?.[1] || !match1[2] || !match1[3] || !match1[4]) {
    throw new Error(`Invalid range format: ${range1}`);
  }
  const startCol1 = colToNum(match1[1]);
  const startRow1 = parseInt(match1[2]);
  const endCol1 = colToNum(match1[3]);
  const endRow1 = parseInt(match1[4]);

  // Parse range2
  const match2 = range2.match(RANGE_REGEX);
  if (!match2?.[1] || !match2[2] || !match2[3] || !match2[4]) {
    throw new Error(`Invalid range format: ${range2}`);
  }
  const startCol2 = colToNum(match2[1]);
  const startRow2 = parseInt(match2[2]);
  const endCol2 = colToNum(match2[3]);
  const endRow2 = parseInt(match2[4]);

  // Check for overlap using bounding box intersection
  // Two rectangles overlap if they overlap on both x and y axes
  return !(endCol1 < startCol2 || endCol2 < startCol1 || endRow1 < startRow2 || endRow2 < startRow1);
};



const UnitConversionSidebar = React.memo<UnitConversionSidebarProps>(({
  open,
  onClose,
  univerRef,
  onSelectionChange,
  category,
  setCategory,
  fromUnit,
  setFromUnit,
  toUnit,
  setToUnit,
  value,
  setValue
}) => {
  const [categories, setCategories] = useState<string[]>([]);
  const [availableUnits, setAvailableUnits] = useState<Record<string, UnitInfo>>({});

  const [searchQuery, setSearchQuery] = useState<string>('');
  const [activeUnitInput, setActiveUnitInput] = useState<'from' | 'to' | null>(null);
  const [lastFocusedUnitInput, setLastFocusedUnitInput] = useState<'from' | 'to'>('from');

  const [outputTarget, setOutputTarget] = useState<string>('');
  const [result, setResult] = useState<string>('');

  // Memoized input type detection
  const getInputType = useCallback((input: string): 'number' | 'cell' | 'range' | 'empty' => {
    if (!input) { return 'empty'; }
    if (input.includes(':')) { return 'range'; }
    if (CELL_REGEX.test(input)) { return 'cell'; }

    // Strict numeric validation: trim input and test against numeric pattern
    const trimmed = input.trim();
    const numericRegex = /^[+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?$/;
    if (numericRegex.test(trimmed)) {
      // Additional check: ensure the entire string is consumed by Number parsing
      const numValue = Number(trimmed);
      if (!isNaN(numValue) && isFinite(numValue)) {
        return 'number';
      }
    }

    throw new Error(`Invalid input format: ${input}`);
  }, []);

  // Optimized unit filtering with useMemo
  const filteredUnitsComputed = useMemo(() => {
    if (!category || Object.keys(availableUnits).length === 0) {
      return [];
    }

    const categoryUnits = Object.keys(availableUnits).filter(
      unit => availableUnits[unit]?.category === category || category === 'All'
    );

    if (!searchQuery) {
      return categoryUnits;
    }

    const query = searchQuery.toLowerCase();
    return categoryUnits.filter(unit => {
      const unitInfo = availableUnits[unit];
      return unitInfo && (
        unit.toLowerCase().includes(query) ||
        unitInfo.name.toLowerCase().includes(query) ||
        unitInfo.description.toLowerCase().includes(query)
      );
    });
  }, [searchQuery, availableUnits, category]);

  // Use the spreadsheet selection hook
  const { focusedInput, handleInputFocus, handleInputBlur } = useSpreadsheetSelection<FocusedInputType>({
    onSelectionChange: onSelectionChange ?? (() => { }),
    updateField: (inputType, selection) => {
      switch (inputType) {
        case 'value':
          setValue(selection);
          break;
        case 'outputTarget':
          setOutputTarget(selection);
          break;
      }
    },
    sidebarDataAttribute: 'data-unit-converter-sidebar',
    handlerName: '__unitConverterSelectionHandler'
  });

  const [error, setError] = useState<string>('');
  const [isConverting, setIsConverting] = useState<boolean>(false);
  const [compatibilityError, setCompatibilityError] = useState<string>('');

  // Subscribe to spreadsheet selection events via event bus
  useEffect(() => {
    if (!open) { return; }

    const unsubscribe = spreadsheetEventBus.on('selection-change', (cellRef) => {
      // Call the window handler that the hook is listening to
      const handler = (window as unknown as Record<string, unknown>).__unitConverterSelectionHandler;
      if (typeof handler === 'function') {
        (handler as (cellRef: string) => void)(cellRef);
      }
    });

    return unsubscribe;
  }, [open]);

  // Define loadCategories before it's used in useEffect
  const loadCategories = useCallback(async () => {
    try {
      const cats: string[] = await invoke('get_supported_categories');
      // Add "All" category at the beginning
      const allCategories = ['All', ...cats];
      setCategories(allCategories);
      if (allCategories.length > 0 && !category) {
        const firstCategory = allCategories[0];
        if (firstCategory) {
          setCategory(firstCategory);
        }
      }
    } catch (err) {
      setError('Failed to load categories');
      console.error(err);
    }
  }, [category, setCategory]);

  // Define loadUnits before it's used in useEffect
  const loadUnits = useCallback(async () => {
    try {
      if (!cacheInitialized) {
        const units: Record<string, UnitInfo> = await invoke('get_available_units');
        unitMetadataCache.clear();
        Object.entries(units).forEach(([symbol, info]) => {
          unitMetadataCache.set(symbol, info);
        });
        cacheInitialized = true;
      }

      const units: Record<string, UnitInfo> = {};
      unitMetadataCache.forEach((info, symbol) => {
        units[symbol] = info;
      });
      setAvailableUnits(units);
    } catch (err) {
      setError('Failed to load units');
      console.error(err);
    }
  }, [setAvailableUnits, setError]);

  // Category icons mapping (blue theme)
  const categoryIcons: Record<string, string> = {
    'All': '🌍',
    'length': '📏',
    'mass': '⚖️',
    'time': '⏱️',
    'temperature': '🌡️',
    'energy': '⚡',
    'power': '💡',
    'pressure': '🔽',
    'velocity': '🚀',
    'force': '💪',
    'frequency': '〰️',
    'current': '⚡',
    'other': '📦'
  };

  // Special symbols for unit entry
  const specialSymbols = [
    { symbol: 'μ', label: 'micro (μ)' },
    { symbol: '°', label: 'degree (°)' },
    { symbol: '²', label: 'squared (²)' },
    { symbol: '³', label: 'cubed (³)' },
    { symbol: '⁴', label: 'power 4 (⁴)' },
    { symbol: '⁻¹', label: 'inverse (⁻¹)' },
    { symbol: '⁻²', label: 'inverse squared (⁻²)' },
    { symbol: 'Ω', label: 'ohm (Ω)' },
  ];

  // Load categories on mount
  useEffect(() => {
    if (open) {
      void loadCategories();
    }
  }, [open, loadCategories]);

  // Load units when category changes
  useEffect(() => {
    if (category) {
      void loadUnits();
    }
  }, [category, loadUnits]);

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

  useEffect(() => {
    if (fromUnit && toUnit && fromUnit !== toUnit) {
      void checkCompatibility();
    } else {
      setCompatibilityError('');
    }
  }, [fromUnit, toUnit, checkCompatibility]);

  // Define all handler functions first before they're used
  const handleSidebarDisplay = useCallback(async (input: string, inputType: string) => {
    let numValue: number;

    if (inputType === 'number') {
      numValue = parseFloat(input);
    } else { // inputType === 'cell'
      if (!univerRef?.current) {
        throw new Error('Spreadsheet not available');
      }
      const cellValue = await univerRef.current.getCellValue(input);
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
  }, [fromUnit, toUnit, value, univerRef]);

  const handleInPlaceReplacement = useCallback(async (cellRef: string) => {
    if (!univerRef?.current) {
      throw new Error('Spreadsheet not available');
    }
    const cellValue = await univerRef.current.getCellValue(cellRef);

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
    await univerRef.current.updateCell(cellRef, { v: convertedValue });

    setResult(`Replaced ${cellRef}: ${numValue} ${fromUnit} → ${convertedValue.toFixed(6)} ${toUnit} (factor: ${conversionFactor.toFixed(6)})`);
  }, [fromUnit, toUnit, univerRef]);

  const handleFillOutput = useCallback(async (fromInput: string, toInput: string, fromType: string, toType: string) => {
    // Get the value to convert
    let numValue: number;
    if (fromType === 'number') {
      numValue = parseFloat(fromInput);
    } else { // fromType === 'cell'
      if (!univerRef?.current) {
        throw new Error('Spreadsheet not available');
      }
      const cellValue = await univerRef.current.getCellValue(fromInput);
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
    if (!univerRef?.current) {
      throw new Error('Spreadsheet not available');
    }

    if (toType === 'cell') {
      // Single cell output
      await univerRef.current.updateCell(toInput, { v: conversionResult.value });
    } else { // toType === 'range'
      // Range output - use efficient range update instead of individual cells
      const rangeFormat = getRangeFormat(toInput);
      // Create 2D array filled with the converted value
      const values: CellValue[][] = Array.from({ length: rangeFormat.rows }, () =>
        Array.from({ length: rangeFormat.cols }, () => ({ v: conversionResult.value }))
      );
      await univerRef.current.updateRange(toInput, values);
    }

    setResult(`Filled ${toType === 'cell' ? '1 cell' : `${parseRange(toInput).length} cells`} with ${conversionResult.value.toFixed(6)} ${toUnit}`);
  }, [fromUnit, toUnit, univerRef]);

  const handleSameRangeConversion = useCallback(async (range: string) => {
    // Get conversion factor
    const conversionResult: ConversionResult = await invoke('convert_value', {
      request: { value: 1, from_unit: fromUnit, to_unit: toUnit }
    });
    const conversionFactor = conversionResult.value;

    if (!univerRef?.current) {
      throw new Error('Spreadsheet reference not available');
    }

    // OPTIMIZED: Read entire range at once
    const rangeData = await univerRef.current.getRange(range);

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

    // OPTIMIZED: Single range update instead of individual cell updates
    await univerRef.current.updateRange(range, convertedData);

    const totalCells = rangeData.flat().length;
    const convertedCount = convertedData.flat().filter(cell =>
      typeof cell.v === 'number' && !isNaN(cell.v)
    ).length;

    setResult(`Applied conversion factor ${conversionFactor.toFixed(6)} to ${convertedCount}/${totalCells} cells in ${range}`);
  }, [fromUnit, toUnit, univerRef]);

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

    if (fromCells.length !== toCells.length) {
      throw new Error(`Range sizes must match: ${fromCells.length} vs ${toCells.length} cells`);
    }

    // Special case: Same range → Apply conversion factor to formulas
    if (fromRange === toRange) {
      await handleSameRangeConversion(fromRange);
      return;
    }

    // OPTIMIZED: Read entire range at once instead of individual cells
    if (!univerRef?.current) {
      throw new Error('Spreadsheet not available');
    }
    const rangeData = await univerRef.current.getRange(fromRange);

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
      await univerRef.current.updateRange(toRange, rangeValues);
    } else {
      // Fallback to individual cell updates for irregular mappings
      for (let i = 0; i < cellMapping.length; i++) {
        const mapping = cellMapping[i];
        if (mapping === undefined) {
          continue;
        }

        const conversionResult = conversionResults[i];
        if (conversionResult !== undefined) {
          await univerRef.current.updateCell(mapping.toCell, { v: conversionResult.value });
        }
      }
    }

    setResult(`Converted ${values.length} values from ${fromRange} to ${toRange}`);
  }, [fromUnit, toUnit, univerRef, handleSameRangeConversion]);

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

  const handleConvert = useCallback(async () => {
    setError('');
    setResult('');

    if (!fromUnit || !toUnit) {
      setError('Please enter both units');
      return;
    }

    // Check if Facade API is ready for cell operations
    if (univerRef?.current && !univerRef.current.isFacadeReady()) {
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
  }, [value, outputTarget, fromUnit, toUnit, univerRef, getInputType, applyConversionLogic]);



  if (!open) { return null; }

  return (
    <Box
      data-unit-converter-sidebar
      sx={{ ...sidebarStyles.container, px: 1, pt: 2 }}
    >
      {/* Header */}
      <Box sx={sidebarStyles.header}>
        <Typography sx={sidebarStyles.text.header}>
          Unit Converter
        </Typography>
        <IconButton
          onClick={onClose}
          size="small"
          sx={sidebarStyles.iconButton}
        >
          <CloseIcon />
        </IconButton>
      </Box>

      {/* Main Content */}
      <Box sx={sidebarStyles.contentWrapper}>


        {/* Conversion Panel */}
        <SidebarCard title="Unit Conversion" sx={{ mx: 0.5 }}>
          {/* From Unit */}
          <Box sx={{ mb: 1 }}>
            <Typography variant="caption" sx={{ color: 'rgba(255, 255, 255, 0.7)', fontSize: 12, fontWeight: 600, mb: 0.5, display: 'block' }}>
              FROM
            </Typography>
            <Box sx={{ display: 'flex', gap: 1 }}>
              <TextField
                size="small"
                value={fromUnit}
                onChange={(e) => setFromUnit(e.target.value)}
                onFocus={() => {
                  setActiveUnitInput('from');
                  setLastFocusedUnitInput('from');
                }}
                onBlur={() => setActiveUnitInput(null)}
                placeholder="e.g., m, kg, s"
                sx={{
                  flex: 1,
                  '& .MuiOutlinedInput-root': {
                    bgcolor: activeUnitInput === 'from' ? 'rgba(33, 150, 243, 0.1)' : 'rgba(33, 150, 243, 0.05)',
                    borderRadius: '6px',
                    '& fieldset': { borderColor: activeUnitInput === 'from' ? anafisColors.spreadsheet : 'rgba(33, 150, 243, 0.2)' },
                    '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                    '&.Mui-focused fieldset': { borderColor: anafisColors.spreadsheet },
                    '& input': { color: 'white', fontFamily: 'monospace', fontSize: 12 }
                  }
                }}
              />
            </Box>

            {/* Special Symbols for From Unit */}
            <Box sx={{ mt: 0.5, display: 'flex', flexWrap: 'wrap', gap: 0.3 }}>
              {specialSymbols.map(({ symbol }) => (
                <Chip
                  key={`from-${symbol}`}
                  label={symbol}
                  size="small"
                  onClick={() => setFromUnit(fromUnit + symbol)}
                  sx={{
                    fontSize: 14,
                    height: 26,
                    bgcolor: 'rgba(33, 150, 243, 0.05)',
                    border: '1px solid rgba(33, 150, 243, 0.2)',
                    color: anafisColors.spreadsheet,
                    '&:hover': { bgcolor: 'rgba(33, 150, 243, 0.15)' }
                  }}
                />
              ))}
            </Box>
          </Box>

          {/* Value Input */}
          <Box sx={{ mb: 1 }}>
            <Typography variant="caption" sx={{ color: 'rgba(255, 255, 255, 0.7)', fontSize: 12, fontWeight: 600, mb: 0.5, display: 'block' }}>
              VALUE
              {focusedInput === 'value' && ' ← select on spreadsheet'}
            </Typography>
            <TextField
              fullWidth
              size="small"
              value={value}
              onChange={(e) => setValue(e.target.value)}
              onFocus={() => handleInputFocus('value')}
              onBlur={handleInputBlur}
              placeholder="e.g., 5.2 or A1 or A1:A10"
              sx={{
                '& .MuiOutlinedInput-root': {
                  bgcolor: focusedInput === 'value' ? 'rgba(33, 150, 243, 0.1)' : 'rgba(33, 150, 243, 0.05)',
                  borderRadius: '6px',
                  '& fieldset': { borderColor: focusedInput === 'value' ? anafisColors.spreadsheet : 'rgba(33, 150, 243, 0.2)' },
                  '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                  '&.Mui-focused fieldset': { borderColor: anafisColors.spreadsheet },
                  '& input': { color: 'white', fontFamily: 'monospace', fontSize: 12 }
                }
              }}
            />
          </Box>

          {/* Swap Button */}
          <Box sx={{ display: 'flex', justifyContent: 'center', mb: 1 }}>
            <IconButton
              onClick={() => {
                const temp = fromUnit;
                setFromUnit(toUnit);
                setToUnit(temp);
              }}
              size="small"
              sx={{
                color: anafisColors.spreadsheet,
                '&:hover': { bgcolor: 'rgba(33, 150, 243, 0.1)' }
              }}
            >
              <SwapIcon />
            </IconButton>
          </Box>

          {/* To Unit */}
          <Box sx={{ mb: 1 }}>
            <Typography variant="caption" sx={{ color: 'rgba(255, 255, 255, 0.7)', fontSize: 12, fontWeight: 600, mb: 0.5, display: 'block' }}>
              TO
            </Typography>
            <Box sx={{ display: 'flex', gap: 1 }}>
              <TextField
                size="small"
                value={toUnit}
                onChange={(e) => setToUnit(e.target.value)}
                onFocus={() => {
                  setActiveUnitInput('to');
                  setLastFocusedUnitInput('to');
                }}
                onBlur={() => setActiveUnitInput(null)}
                placeholder="e.g., ft, lb, min"
                sx={{
                  flex: 1,
                  '& .MuiOutlinedInput-root': {
                    bgcolor: activeUnitInput === 'to' ? 'rgba(33, 150, 243, 0.1)' : 'rgba(33, 150, 243, 0.05)',
                    borderRadius: '6px',
                    '& fieldset': { borderColor: activeUnitInput === 'to' ? anafisColors.spreadsheet : 'rgba(33, 150, 243, 0.2)' },
                    '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                    '&.Mui-focused fieldset': { borderColor: anafisColors.spreadsheet },
                    '& input': { color: 'white', fontFamily: 'monospace', fontSize: 12 }
                  }
                }}
              />
            </Box>

            {/* Special Symbols for To Unit */}
            <Box sx={{ mt: 0.5, display: 'flex', flexWrap: 'wrap', gap: 0.3 }}>
              {specialSymbols.map(({ symbol }) => (
                <Chip
                  key={`to-${symbol}`}
                  label={symbol}
                  size="small"
                  onClick={() => setToUnit(toUnit + symbol)}
                  sx={{
                    fontSize: 14,
                    height: 26,
                    bgcolor: 'rgba(33, 150, 243, 0.05)',
                    border: '1px solid rgba(33, 150, 243, 0.2)',
                    color: anafisColors.spreadsheet,
                    '&:hover': { bgcolor: 'rgba(33, 150, 243, 0.15)' }
                  }}
                />
              ))}
            </Box>
          </Box>

          {/* To Field */}
          <Box sx={{ mt: 1 }}>
            <Typography variant="caption" sx={{ color: 'rgba(255, 255, 255, 0.7)', fontSize: 12, fontWeight: 600, mb: 0.5, display: 'block' }}>
              TO (optional - leave empty to show result below)
              {focusedInput === 'outputTarget' && ' ← select on spreadsheet'}
            </Typography>
            <TextField
              fullWidth
              size="small"
              value={outputTarget}
              onChange={(e) => setOutputTarget(e.target.value)}
              onFocus={() => handleInputFocus('outputTarget')}
              onBlur={handleInputBlur}
              placeholder="e.g., B1 or B1:B10 (optional)"
              sx={{
                '& .MuiOutlinedInput-root': {
                  bgcolor: focusedInput === 'outputTarget' ? 'rgba(33, 150, 243, 0.1)' : 'rgba(33, 150, 243, 0.05)',
                  borderRadius: '6px',
                  '& fieldset': { borderColor: focusedInput === 'outputTarget' ? anafisColors.spreadsheet : 'rgba(33, 150, 243, 0.2)' },
                  '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                  '&.Mui-focused fieldset': { borderColor: anafisColors.spreadsheet },
                  '& input': { color: 'white', fontFamily: 'monospace', fontSize: 12 }
                }
              }}
            />
          </Box>

          {/* Convert Button */}
          <Button
            fullWidth
            variant="contained"
            startIcon={isConverting ? <CircularProgress size={16} /> : <ConvertIcon />}
            onClick={() => void handleConvert()}
            disabled={isConverting || !!compatibilityError}
            sx={{
              ...sidebarStyles.button.primary,
              mt: 2,
              fontSize: 14,
              py: 1.5
            }}
          >
            {isConverting ? 'Converting...' : 'Convert'}
          </Button>

          {/* Error Display */}
          {error && (
            <Alert severity="error" sx={{ mt: 2, py: 0.5, fontSize: 12 }}>
              {error}
            </Alert>
          )}

          {/* Result Display */}
          {result && (
            <Box sx={{ mt: 2, p: 1.5, bgcolor: 'rgba(33, 150, 243, 0.1)', borderRadius: 1, border: '1px solid rgba(33, 150, 243, 0.2)' }}>
              <Typography sx={{ ...sidebarStyles.text.body, fontFamily: 'monospace', fontSize: '0.9rem', color: anafisColors.spreadsheet }}>
                {result}
              </Typography>
            </Box>
          )}

          {/* Compatibility Error Display */}
          {compatibilityError && (
            <Alert severity="error" onClose={() => setCompatibilityError('')} sx={{ mt: 1, py: 0.5, fontSize: 12 }}>
              {compatibilityError}
            </Alert>
          )}
        </SidebarCard>

        {/* Category Selection, Search, and Units List */}
        <Box>
          <SidebarCard variant="compact" title="Category" sx={{ mx: 0.5 }}>
            <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5 }}>
              {categories.map((cat) => (
                <Chip
                  key={cat}
                  label={`${categoryIcons[cat] ?? '📦'}`}
                  size="small"
                  onClick={() => setCategory(cat)}
                  sx={{
                    fontSize: 16,
                    height: 28,
                    bgcolor: category === cat ? 'rgba(33, 150, 243, 0.2)' : 'rgba(33, 150, 243, 0.05)',
                    border: '1px solid',
                    borderColor: category === cat ? anafisColors.spreadsheet : 'rgba(33, 150, 243, 0.2)',
                    color: anafisColors.spreadsheet,
                    '&:hover': { bgcolor: 'rgba(33, 150, 243, 0.15)' }
                  }}
                />
              ))}
            </Box>
          </SidebarCard>

          <SidebarCard variant="compact" title="Search Units" sx={{ mx: 0.5, flex: 1, minHeight: 450 }}>
            <TextField
              fullWidth
              size="small"
              placeholder="Search units..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              InputProps={{
                startAdornment: <SearchIcon sx={{ mr: 1, color: 'text.secondary' }} />,
              }}
              sx={{
                mb: 1,
                '& .MuiOutlinedInput-root': {
                  bgcolor: 'rgba(33, 150, 243, 0.05)',
                  '& fieldset': { borderColor: 'rgba(33, 150, 243, 0.2)' },
                  '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                  '&.Mui-focused fieldset': { borderColor: anafisColors.spreadsheet },
                  '& input': { color: 'white', fontSize: 12 }
                }
              }}
            />

            <List dense sx={{ maxHeight: 450, overflow: 'auto' }}>
              {filteredUnitsComputed.slice(0, 50).map((symbol) => {
                const unitInfo = availableUnits[symbol];
                if (!unitInfo) { return null; }

                return (
                  <ListItemButton
                    key={symbol}
                    onClick={() => {
                      // Use activeUnitInput if available, otherwise use lastFocusedUnitInput
                      const targetInput = activeUnitInput ?? lastFocusedUnitInput;

                      if (targetInput === 'from') {
                        setFromUnit(fromUnit + symbol);
                      } else {
                        setToUnit(toUnit + symbol);
                      }
                    }}
                    sx={{
                      py: 0.3,
                      px: 0.8,
                      borderRadius: 1,
                      mb: 0.3,
                      bgcolor: (fromUnit === symbol || toUnit === symbol) ? 'rgba(33, 150, 243, 0.1)' : 'transparent',
                      '&:hover': { bgcolor: 'rgba(33, 150, 243, 0.15)' }
                    }}
                  >
                    <ListItemText
                      primary={
                        <Typography sx={{ fontSize: 12, fontWeight: 600, color: anafisColors.spreadsheet }}>
                          {symbol}
                        </Typography>
                      }
                      secondary={
                        <Typography sx={{ fontSize: 11, color: 'rgba(255, 255, 255, 0.7)' }}>
                          {unitInfo.name}
                        </Typography>
                      }
                    />
                  </ListItemButton>
                );
              })}
            </List>
          </SidebarCard>
        </Box>
      </Box>
    </Box>
  );
});

UnitConversionSidebar.displayName = 'UnitConversionSidebar';

export default UnitConversionSidebar;