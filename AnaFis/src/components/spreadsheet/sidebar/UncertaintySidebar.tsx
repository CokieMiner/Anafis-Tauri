import React, { useState, useMemo, useCallback } from 'react';
import { Box, Typography, IconButton, List, ListItemButton, ListItemText, TextField, Button } from '@mui/material';
import { Close as CloseIcon, Add as AddIcon, Delete as DeleteIcon, PlayArrow as RunIcon } from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';
import { SpreadsheetRef } from '../SpreadsheetInterface';
import { useSpreadsheetSelection } from '@/hooks/useSpreadsheetSelection';
import { sidebarStyles } from '@/utils/sidebarStyles';
import SidebarCard from './SidebarCard';
import { anafisColors } from '@/themes';
import { spreadsheetEventBus } from '../SpreadsheetEventBus';
import { normalizeRangeRef } from '../univer/errors';

interface Variable {
  name: string;
  valueRange: string;
  uncertaintyRange: string;
  confidence: number;
}

// Helper types for optimized validation (moved outside component)
interface RangeRequest {
  type: 'value' | 'uncertainty';
  variableIndex: number;
  range: string;
  variableName: string;
}

interface RangeResult extends RangeRequest {
  data: (string | number)[][];
}

// Helper functions moved outside component to prevent recreation
const collectRangeRequests = (variables: Variable[]): RangeRequest[] => {
  return variables.flatMap((variable, index) => {
    const requests: RangeRequest[] = [];

    if (variable.valueRange) {
      requests.push({
        type: 'value',
        variableIndex: index,
        range: variable.valueRange,
        variableName: variable.name
      });
    }

    if (variable.uncertaintyRange) {
      requests.push({
        type: 'uncertainty',
        variableIndex: index,
        range: variable.uncertaintyRange,
        variableName: variable.name
      });
    }

    return requests;
  });
};

// Check if data contains only numeric values
const isAllNumericData = (data: (string | number)[][]): boolean => {
  return data.every(row =>
    row.every(cell => typeof cell === 'number' && isFinite(cell))
  );
};

// Validate all range data in batch
const validateRangeData = (rangeResults: RangeResult[], variables: Variable[]): void => {
  // Group results by variable for easier validation
  const resultsByVariable = new Map<number, { value?: RangeResult; uncertainty?: RangeResult }>();

  for (const result of rangeResults) {
    if (!resultsByVariable.has(result.variableIndex)) {
      resultsByVariable.set(result.variableIndex, {});
    }

    const varResults = resultsByVariable.get(result.variableIndex)!;
    if (result.type === 'value') {
      varResults.value = result;
    } else {
      varResults.uncertainty = result;
    }
  }

  // Validate each variable's data
  for (const [varIndex, results] of resultsByVariable) {
    const variable = variables[varIndex]!;

    // Validate value data
    if (results.value) {
      if (!isAllNumericData(results.value.data)) {
        throw new Error(`Variable "${variable.name}" value range contains non-numeric data`);
      }
    }

    // Validate uncertainty data and length matching
    if (results.uncertainty && results.value) {
      if (!isAllNumericData(results.uncertainty.data)) {
        throw new Error(`Variable "${variable.name}" uncertainty range contains non-numeric data`);
      }

      if (results.uncertainty.data.length !== results.value.data.length) {
        throw new Error(`Variable "${variable.name}": uncertainty range length doesn't match value range`);
      }
    }
  }
};

// Generate next variable name: a-z, then aa-zz
const generateNextVariableName = (variableCount: number): string => {
  if (variableCount < 26) {
    // a-z
    return String.fromCharCode(97 + variableCount);
  } else {
    // aa-zz
    const doubleIndex = variableCount - 26;
    const firstChar = String.fromCharCode(97 + Math.floor(doubleIndex / 26));
    const secondChar = String.fromCharCode(97 + (doubleIndex % 26));
    return firstChar + secondChar;
  }
};

// Helper functions for range validation and intersection
interface RangeBounds {
  startCol: number;
  startRow: number;
  endCol: number;
  endRow: number;
}

/**
 * Parse a range string into bounds
 */
function parseRangeBounds(range: string): RangeBounds | null {
  // Handle single cell: "A1"
  const singleCellMatch = range.match(/^([A-Z]+)(\d+)$/);
  if (singleCellMatch) {
    const col = columnToNumber(singleCellMatch[1]!);
    const row = parseInt(singleCellMatch[2]!);
    return { startCol: col, startRow: row, endCol: col, endRow: row };
  }

  // Handle range: "A1:B5"
  const rangeMatch = range.match(/^([A-Z]+)(\d+):([A-Z]+)(\d+)$/);
  if (rangeMatch) {
    const startCol = columnToNumber(rangeMatch[1]!);
    const startRow = parseInt(rangeMatch[2]!);
    const endCol = columnToNumber(rangeMatch[3]!);
    const endRow = parseInt(rangeMatch[4]!);
    return { startCol, startRow, endCol, endRow };
  }

  return null;
}

/**
 * Convert column letter to number (A=1, B=2, ..., Z=26, AA=27, etc.)
 */
function columnToNumber(col: string): number {
  let result = 0;
  for (let i = 0; i < col.length; i++) {
    result = result * 26 + (col.charCodeAt(i) - 64);
  }
  return result;
}

/**
 * Check if two ranges intersect/overlap
 */
function rangesIntersect(range1: RangeBounds, range2: RangeBounds): boolean {
  return !(range1.endCol < range2.startCol ||
           range2.endCol < range1.startCol ||
           range1.endRow < range2.startRow ||
           range2.endRow < range1.startRow);
}

/**
 * Validate output ranges comprehensively
 */
async function validateOutputRanges(
  outputValueRange: string,
  outputUncertaintyRange: string,
  variables: Variable[],
  univerAPI: SpreadsheetRef
): Promise<void> {
  // 1. Validate range formats
  if (!outputValueRange.trim()) {
    throw new Error('Output value range is required');
  }
  if (!outputUncertaintyRange.trim()) {
    throw new Error('Output uncertainty range is required');
  }

  try {
    normalizeRangeRef(outputValueRange);
  } catch (_error) {
    throw new Error(`Invalid output value range format: ${outputValueRange}`);
  }

  try {
    normalizeRangeRef(outputUncertaintyRange);
  } catch (_error) {
    throw new Error(`Invalid output uncertainty range format: ${outputUncertaintyRange}`);
  }

  // 2. Parse ranges into bounds
  const valueBounds = parseRangeBounds(outputValueRange);
  const uncertaintyBounds = parseRangeBounds(outputUncertaintyRange);

  if (!valueBounds) {
    throw new Error(`Could not parse output value range: ${outputValueRange}`);
  }
  if (!uncertaintyBounds) {
    throw new Error(`Could not parse output uncertainty range: ${outputUncertaintyRange}`);
  }

  // 3. Check for self-intersection between output ranges
  if (rangesIntersect(valueBounds, uncertaintyBounds)) {
    throw new Error('Output value and uncertainty ranges cannot overlap');
  }

  // 4. Collect all input ranges for overlap checking
  const inputRanges: RangeBounds[] = [];
  for (const variable of variables) {
    if (variable.valueRange) {
      const bounds = parseRangeBounds(variable.valueRange);
      if (bounds) {
        inputRanges.push(bounds);
      }
    }
    if (variable.uncertaintyRange) {
      const bounds = parseRangeBounds(variable.uncertaintyRange);
      if (bounds) {
        inputRanges.push(bounds);
      }
    }
  }

  // 5. Check output ranges don't overlap with input ranges
  for (const inputRange of inputRanges) {
    if (rangesIntersect(valueBounds, inputRange)) {
      throw new Error(`Output value range "${outputValueRange}" overlaps with input data range`);
    }
    if (rangesIntersect(uncertaintyBounds, inputRange)) {
      throw new Error(`Output uncertainty range "${outputUncertaintyRange}" overlaps with input data range`);
    }
  }

  // 6. Verify ranges exist and are within sheet bounds by attempting to read them
  try {
    await univerAPI.getRange(outputValueRange);
  } catch (_error) {
    throw new Error(`Output value range "${outputValueRange}" is not accessible or out of bounds`);
  }

  try {
    await univerAPI.getRange(outputUncertaintyRange);
  } catch (_error) {
    throw new Error(`Output uncertainty range "${outputUncertaintyRange}" is not accessible or out of bounds`);
  }

  // 7. Check if ranges are writable (attempt to read and see if we get data)
  // This is a basic check - more sophisticated protection checking would require
  // additional API support for protected ranges
  try {
    const testData = await univerAPI.getRange(outputValueRange);
    if (!Array.isArray(testData) || testData.length === 0) {
      throw new Error(`Output value range "${outputValueRange}" appears to be empty or inaccessible`);
    }
  } catch (_error) {
    throw new Error(`Output value range "${outputValueRange}" is not writable`);
  }

  try {
    const testData = await univerAPI.getRange(outputUncertaintyRange);
    if (!Array.isArray(testData) || testData.length === 0) {
      throw new Error(`Output uncertainty range "${outputUncertaintyRange}" appears to be empty or inaccessible`);
    }
  } catch (_error) {
    throw new Error(`Output uncertainty range "${outputUncertaintyRange}" is not writable`);
  }

  // Note: Merged cell handling would require additional API support
  // For now, we assume the ranges are valid for writing
}

type FocusedInputType =
  | { type: 'valueRange'; varIndex: number }
  | { type: 'uncertaintyRange'; varIndex: number }
  | { type: 'outputValueRange' }
  | { type: 'outputUncertaintyRange' }
  | null;

interface UncertaintySidebarProps {
  open: boolean;
  onClose: () => void;
  univerRef: React.RefObject<SpreadsheetRef | null>;
  onSelectionChange?: (selection: string) => void;
  onPropagationComplete?: (resultRange: string) => void;
  // Lifted state
  variables: Variable[];
  setVariables: (vars: Variable[]) => void;
  formula: string;
  setFormula: (formula: string) => void;
  outputValueRange: string;
  setOutputValueRange: (range: string) => void;
  outputUncertaintyRange: string;
  setOutputUncertaintyRange: (range: string) => void;
  outputConfidence: number;
  setOutputConfidence: (confidence: number) => void;
}

export const UncertaintySidebar = React.memo<UncertaintySidebarProps>(({
  open,
  onClose,
  univerRef,
  onSelectionChange,
  onPropagationComplete,
  variables,
  setVariables,
  formula,
  setFormula,
  outputValueRange,
  setOutputValueRange,
  outputUncertaintyRange,
  setOutputUncertaintyRange,
  outputConfidence,
  setOutputConfidence
}) => {
  // Remove local state - now using props
  const [selectedVariable, setSelectedVariable] = useState<number>(0);
  const [isProcessing, setIsProcessing] = useState<boolean>(false);
  const [error, setError] = useState<string>('');

  // Memoized current variable for performance
  const currentVariable = useMemo(() =>
    variables[selectedVariable],
    [variables, selectedVariable]
  );

  // Use the spreadsheet selection hook
  const { handleInputFocus, handleInputBlur } = useSpreadsheetSelection<FocusedInputType>({
    onSelectionChange: onSelectionChange ?? (() => { }),
    updateField: (inputType, selection) => {
      if (!inputType) { return; }

      switch (inputType.type) {
        case 'valueRange':
          updateVariable(inputType.varIndex, 'valueRange', selection);
          break;
        case 'uncertaintyRange':
          updateVariable(inputType.varIndex, 'uncertaintyRange', selection);
          break;
        case 'outputValueRange':
          setOutputValueRange(selection);
          break;
        case 'outputUncertaintyRange':
          setOutputUncertaintyRange(selection);
          break;
      }
    },
    sidebarDataAttribute: 'data-uncertainty-sidebar',
    handlerName: '__uncertaintySidebarSelectionHandler'
  });

  // Subscribe to spreadsheet selection events via event bus
  React.useEffect(() => {
    if (!open) { return; }

    const unsubscribe = spreadsheetEventBus.on('selection-change', (cellRef) => {
      // Call the window handler that the hook is listening to
      const handler = (window as unknown as Record<string, (cellRef: string) => void>).__uncertaintySidebarSelectionHandler;
      if (handler) {
        handler(cellRef);
      }
      // NOTE: Don't call onSelectionChange here - it would create an infinite loop
      // since onSelectionChange emits to the event bus, which triggers this handler again
    });

    return unsubscribe;
  }, [open]);

  // Memoized parseRange function to avoid regex recompilation
  const parseRange = useCallback((range: string): { col: string; row: number } | null => {
    // Try range format first: "A1:B5"
    let match = range.match(/^([A-Z]+)(\d+):([A-Z]+)(\d+)$/);
    if (match?.[1] && match[2]) {
      return { col: match[1], row: parseInt(match[2]) };
    }

    // Try single cell format: "A1"
    match = range.match(/^([A-Z]+)(\d+)$/);
    if (match?.[1] && match[2]) {
      return { col: match[1], row: parseInt(match[2]) };
    }

    return null;
  }, []);

  const addVariable = useCallback(() => {
    const nextName = generateNextVariableName(variables.length);
    const newVariable: Variable = {
      name: nextName,
      valueRange: '',
      uncertaintyRange: '',
      confidence: 95
    };
    setVariables([...variables, newVariable]);
    setSelectedVariable(variables.length);
  }, [variables, setVariables]);

  const removeVariable = useCallback((index: number) => {
    if (variables.length > 1) {
      setVariables(variables.filter((_, i) => i !== index));
      setSelectedVariable(index > 0 ? index - 1 : 0);
    }
  }, [variables, setVariables]);

  const updateVariable = useCallback((index: number, field: keyof Variable, value: string | number) => {
    const updated = [...variables];
    const currentVar = updated[index];
    if (currentVar) {
      updated[index] = { ...currentVar, [field]: value } as Variable;
      setVariables(updated);
    }
  }, [variables, setVariables]);



  // Optimized parallel validation (memoized)
  const validateRanges = useCallback(async (): Promise<boolean> => {
    if (!univerRef.current) {
      setError('Spreadsheet not initialized');
      return false;
    }

    // Capture the current API instance to avoid race conditions
    const univerAPI = univerRef.current;

    try {
      // First validate input ranges (existing logic)
      const rangeRequests = collectRangeRequests(variables);

      if (rangeRequests.length === 0) {
        setError('No input ranges to validate');
        return false;
      }

      // Performance monitoring
      const startTime = performance.now();
      if (process.env.NODE_ENV === 'development') {
        console.log(`[UncertaintySidebar] Starting parallel validation of ${rangeRequests.length} input ranges`);
      }

      // Read all input ranges in parallel
      const rangeResults = await Promise.all(
        rangeRequests.map(async (request) => {
          try {
            const data = await univerAPI.getRange(request.range);
            return { ...request, data };
          } catch (error) {
            throw new Error(`Failed to read ${request.type} range "${request.range}" for variable "${request.variableName}": ${String(error)}`);
          }
        })
      );

      // Validate all input data in batch
      validateRangeData(rangeResults, variables);

      // Now validate output ranges comprehensively
      if (process.env.NODE_ENV === 'development') {
        console.log(`[UncertaintySidebar] Validating output ranges: "${outputValueRange}" and "${outputUncertaintyRange}"`);
      }

      await validateOutputRanges(outputValueRange, outputUncertaintyRange, variables, univerAPI);

      // Performance logging
      if (process.env.NODE_ENV === 'development') {
        const endTime = performance.now();
        console.log(`[UncertaintySidebar] Complete validation completed in ${(endTime - startTime).toFixed(2)}ms`);
      }

      return true;

    } catch (err) {
      setError(`Validation failed: ${String(err)}`);
      return false;
    }
  }, [variables, univerRef, outputValueRange, outputUncertaintyRange]);

  const handlePropagate = useCallback(async () => {
    setError('');
    if (variables.some(v => !v.valueRange)) {
      setError('Fill in all value ranges');
      return;
    }
    if (!formula || !outputValueRange || !outputUncertaintyRange) {
      setError('Fill in formula and output ranges');
      return;
    }

    if (!univerRef.current) {
      setError('Spreadsheet not initialized');
      return;
    }

    setIsProcessing(true);
    try {
      // Validate data before sending to backend
      const isValid = await validateRanges();
      if (!isValid) {
        setIsProcessing(false);
        return;
      }
      // Call backend to generate formulas
      const result = await invoke<{
        value_formulas: string[];
        uncertainty_formulas: string[];
        success: boolean;
        error?: string;
      }>('generate_uncertainty_formulas', {
        variables: variables.map(v => ({
          name: v.name,
          value_range: v.valueRange,
          uncertainty_range: v.uncertaintyRange,
          confidence: v.confidence
        })),
        formula,
        outputConfidence
      });

      if (!result.success || result.error) {
        setError(result.error ?? 'Formula generation failed');
        return;
      }

      // Parse output ranges to get starting cell (using memoized function)
      const valueStart = parseRange(outputValueRange);
      const uncStart = parseRange(outputUncertaintyRange);

      if (!valueStart || !uncStart) {
        setError('Invalid output range format');
        return;
      }

      // Prepare batch updates for value formulas
      const valueUpdates = result.value_formulas.map((formula, i) => ({
        cellRef: `${valueStart.col}${valueStart.row + i}`,
        value: { f: formula }
      }));

      // Prepare batch updates for uncertainty formulas
      const uncertaintyUpdates = result.uncertainty_formulas.map((formula, i) => ({
        cellRef: `${uncStart.col}${uncStart.row + i}`,
        value: { f: formula }
      }));

      // Write all formulas in batches (much faster than one-by-one)
      if (process.env.NODE_ENV === 'development') {
        console.log(`Writing ${valueUpdates.length} value formulas and ${uncertaintyUpdates.length} uncertainty formulas in batch`);
      }

      await univerRef.current.batchUpdateCells([...valueUpdates, ...uncertaintyUpdates]);

      onPropagationComplete?.(outputValueRange);
      setError('');
    } catch (err: unknown) {
      console.error('Propagation error:', err);
      setError(String(err));
    } finally {
      setIsProcessing(false);
    }
  }, [variables, formula, outputValueRange, outputUncertaintyRange, outputConfidence, univerRef, validateRanges, parseRange, onPropagationComplete]);

  if (!open) { return null; }

  // Return early if no current variable
  if (!currentVariable) { return null; }

  return (
    <Box
      data-uncertainty-sidebar
      sx={{ ...sidebarStyles.container, px: 1, pt: 2 }}
    >
      {/* Header */}
      <Box sx={sidebarStyles.header}>
        <Typography sx={sidebarStyles.text.header}>
          Uncertainty Propagation
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
      <Box sx={{ flex: 1, display: 'flex', overflow: 'hidden', gap: 1.5, p: 1.5 }}>
        {/* Variables List */}
        <SidebarCard title="Variables" sx={{ width: 140, flexShrink: 0, mx: 0.5 }}>
          <Button
            fullWidth
            size="small"
            startIcon={<AddIcon sx={{ fontSize: 16 }} />}
            onClick={addVariable}
            sx={sidebarStyles.button.secondary}
          >
            Add Variable
          </Button>

          <List dense sx={{ mt: 1.5 }}>
            {variables.map((variable, index) => (
              <ListItemButton
                key={index}
                selected={selectedVariable === index}
                onClick={() => setSelectedVariable(index)}
                sx={{
                  px: 1,
                  py: 0.75,
                  mb: 0.5,
                  borderRadius: '6px',
                  border: selectedVariable === index ? `1px solid ${anafisColors.spreadsheet}` : '1px solid rgba(255, 255, 255, 0.2)',
                  bgcolor: selectedVariable === index ? 'rgba(33, 150, 243, 0.15)' : 'transparent',
                  color: selectedVariable === index ? '#ffffff' : 'rgba(255, 255, 255, 0.7)',
                  transition: 'all 0.2s',
                  '&:hover': {
                    bgcolor: selectedVariable === index ? 'rgba(33, 150, 243, 0.2)' : 'rgba(255, 255, 255, 0.05)',
                    borderColor: selectedVariable === index ? anafisColors.spreadsheet : 'rgba(255, 255, 255, 0.4)',
                    color: '#ffffff',
                    transform: 'translateY(-1px)',
                    boxShadow: selectedVariable === index ? `0 2px 8px rgba(33, 150, 243, 0.3)` : '0 2px 8px rgba(255, 255, 255, 0.1)'
                  },
                  '&.Mui-selected': {
                    bgcolor: 'rgba(33, 150, 243, 0.15) !important',
                    borderColor: `${anafisColors.spreadsheet} !important`,
                    color: '#ffffff !important',
                    '&:hover': {
                      bgcolor: 'rgba(33, 150, 243, 0.2) !important'
                    }
                  }
                }}
              >
                <ListItemText
                  primary={
                    <Box sx={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 0.25 }}>
                      <Typography component="span" sx={{ fontSize: 18, fontFamily: 'monospace', fontWeight: 600 }}>
                        {variable.name}
                      </Typography>
                      <Typography variant="body2" sx={{ fontSize: 9, fontWeight: 500, textAlign: 'center', lineHeight: 1.2 }}>
                        variable
                      </Typography>
                    </Box>
                  }
                />
              </ListItemButton>
            ))}
          </List>
        </SidebarCard>
        {/* Variable Configuration */}
        <Box sx={{ flex: 1, display: 'flex', flexDirection: 'column', gap: 1.5 }}>
          {/* Variable Details */}
          <SidebarCard title={`Variable ${currentVariable.name}`} sx={{ mx: 0.5 }}>
            <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 1.5 }}>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, flex: 1 }}>
                <Typography sx={{ ...sidebarStyles.text.label, minWidth: 'fit-content' }}>
                  Name:
                </Typography>
                <TextField
                  value={currentVariable.name}
                  onChange={(e) => updateVariable(selectedVariable, 'name', e.target.value)}
                  size="small"
                  placeholder="a"
                  sx={{
                    maxWidth: 80,
                    ...sidebarStyles.input
                  }}
                  inputProps={{
                    style: {
                      color: anafisColors.spreadsheet,
                      fontFamily: 'monospace',
                      fontSize: 14,
                      fontWeight: 600,
                      textAlign: 'center',
                      padding: '4px 8px'
                    }
                  }}
                />
              </Box>
              {variables.length > 1 && (
                <IconButton
                  onClick={() => removeVariable(selectedVariable)}
                  size="small"
                  sx={{
                    color: '#f44336',
                    borderRadius: '6px',
                    '&:hover': {
                      bgcolor: 'rgba(244, 67, 54, 0.1)',
                      transform: 'scale(1.1)'
                    }
                  }}
                >
                  <DeleteIcon fontSize="small" />
                </IconButton>
              )}
            </Box>

            <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1.5 }}>
              <TextField
                label="Value Range"
                value={currentVariable.valueRange}
                onChange={(e) => updateVariable(selectedVariable, 'valueRange', e.target.value)}
                onFocus={() => handleInputFocus({ type: 'valueRange', varIndex: selectedVariable })}
                onBlur={handleInputBlur}
                placeholder="A1 or A1:A10"
                size="small"
                fullWidth
                sx={sidebarStyles.input}
                slotProps={{
                  input: { style: { color: 'white', fontFamily: 'monospace', fontSize: 12 } },
                  inputLabel: { style: { color: 'rgba(255,255,255,0.7)', fontSize: 12 } }
                }}
              />
              <TextField
                label="Uncertainty Range (optional)"
                value={currentVariable.uncertaintyRange}
                onChange={(e) => updateVariable(selectedVariable, 'uncertaintyRange', e.target.value)}
                onFocus={() => handleInputFocus({ type: 'uncertaintyRange', varIndex: selectedVariable })}
                onBlur={handleInputBlur}
                placeholder="B1 or B1:B10 or leave empty for zero"
                size="small"
                fullWidth
                sx={sidebarStyles.input}
                slotProps={{
                  input: { style: { color: 'white', fontFamily: 'monospace', fontSize: 12 } },
                  inputLabel: { style: { color: 'rgba(255,255,255,0.7)', fontSize: 12 } }
                }}
              />
              <TextField
                label="Confidence (%)"
                type="number"
                value={currentVariable.confidence}
                onChange={(e) => updateVariable(selectedVariable, 'confidence', Number(e.target.value))}
                placeholder="95"
                size="small"
                fullWidth
                sx={sidebarStyles.input}
                slotProps={{
                  input: {
                    style: { color: 'white', fontFamily: 'monospace', fontSize: 12 },
                    inputProps: { min: 50, max: 99.9, step: 0.1 }
                  },
                  inputLabel: { style: { color: 'rgba(255,255,255,0.7)', fontSize: 12 } }
                }}
              />
            </Box>
          </SidebarCard>

          {/* Formula */}
          <SidebarCard title="Formula" sx={{ mx: 0.5 }}>
            <TextField
              value={formula}
              onChange={(e) => setFormula(e.target.value)}
              placeholder={`Variables: ${variables.map(v => v.name).join(', ')}`}
              multiline
              rows={2}
              fullWidth
              sx={sidebarStyles.input}
              slotProps={{
                input: { style: { color: 'white', fontFamily: 'monospace', fontSize: 13 } }
              }}
            />
            <Typography sx={{ ...sidebarStyles.text.caption, mt: 0.5 }}>
              Examples: x+y, x*y^2, sqrt(x^2+y^2)
            </Typography>
          </SidebarCard>

          {/* Output */}
          <SidebarCard title="Output" sx={{ mx: 0.5 }}>
            <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1.5 }}>
              <TextField
                label="Result Values"
                value={outputValueRange}
                onChange={(e) => setOutputValueRange(e.target.value)}
                onFocus={() => handleInputFocus({ type: 'outputValueRange' })}
                onBlur={handleInputBlur}
                placeholder="C1 or C1:C10"
                size="small"
                fullWidth
                sx={sidebarStyles.input}
                slotProps={{
                  input: { style: { color: 'white', fontFamily: 'monospace', fontSize: 12 } },
                  inputLabel: { style: { color: 'rgba(255,255,255,0.7)', fontSize: 12 } }
                }}
              />
              <TextField
                label="Result Uncertainties"
                value={outputUncertaintyRange}
                onChange={(e) => setOutputUncertaintyRange(e.target.value)}
                onFocus={() => handleInputFocus({ type: 'outputUncertaintyRange' })}
                onBlur={handleInputBlur}
                placeholder="D1 or D1:D10"
                size="small"
                fullWidth
                sx={sidebarStyles.input}
                slotProps={{
                  input: { style: { color: 'white', fontFamily: 'monospace', fontSize: 12 } },
                  inputLabel: { style: { color: 'rgba(255,255,255,0.7)', fontSize: 12 } }
                }}
              />
              <TextField
                label="Output Confidence (%)"
                type="number"
                value={outputConfidence}
                onChange={(e) => setOutputConfidence(Number(e.target.value))}
                placeholder="95"
                size="small"
                fullWidth
                sx={sidebarStyles.input}
                slotProps={{
                  input: {
                    style: { color: 'white', fontFamily: 'monospace', fontSize: 12 },
                    inputProps: { min: 50, max: 99.9, step: 0.1 }
                  },
                  inputLabel: { style: { color: 'rgba(255,255,255,0.7)', fontSize: 12 } }
                }}
              />
              <Button
                fullWidth
                variant="contained"
                startIcon={<RunIcon />}
                onClick={() => void handlePropagate()}
                disabled={isProcessing}
                sx={sidebarStyles.button.primary}
              >
                {isProcessing ? 'Processing...' : 'Propagate'}
              </Button>
            </Box>

            {error && (
              <Box sx={{
                mt: 1.5,
                p: 1,
                bgcolor: 'rgba(244, 67, 54, 0.1)',
                borderRadius: '6px',
                border: '1px solid rgba(244, 67, 54, 0.3)'
              }}>
                <Typography sx={{ ...sidebarStyles.text.caption, color: '#f44336' }}>
                  {error}
                </Typography>
              </Box>
            )}
          </SidebarCard>
        </Box>
      </Box>
    </Box>
  );
});

UncertaintySidebar.displayName = 'UncertaintySidebar';

export default UncertaintySidebar;
