// uncertaintyOperations.ts - High-level uncertainty propagation operations
import { SpreadsheetRef } from '@/components/spreadsheet/SpreadsheetInterface';
import { parseRange, rangesIntersect, type RangeBounds } from '../utils/univerUtils';
import { columnToLetter } from '../utils/cellUtils';
import { normalizeRangeRef } from '../utils/validation';

// Types for uncertainty operations
export interface Variable {
  name: string;
  valueRange: string;
  uncertaintyRange: string;
  confidence: number;
}

export interface ValidationResult {
  isValid: boolean;
  error?: string;
}

export interface PropagationResult {
  success: boolean;
  error?: string;
  valueFormulas?: string[];
  uncertaintyFormulas?: string[];
}

interface RangeRequest {
  type: 'value' | 'uncertainty';
  variableIndex: number;
  range: string;
  variableName: string;
}

interface RangeResult extends RangeRequest {
  data: (string | number)[][];
}

/**
 * Parse a range string into bounds using centralized parseRange
 */
function parseRangeBounds(range: string): RangeBounds | null {
  return parseRange(range);
}

/**
 * Check if data contains only numeric values
 */
function isAllNumericData(data: (string | number)[][]): boolean {
  return data.every(row =>
    row.every(cell => typeof cell === 'number' && isFinite(cell))
  );
}

/**
 * Collect all range requests for parallel processing
 */
function collectRangeRequests(variables: Variable[]): RangeRequest[] {
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
}

/**
 * Validate all range data in batch
 */
function validateRangeData(rangeResults: RangeResult[], variables: Variable[]): void {
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
}

/**
 * Validate output ranges comprehensively
 */
async function validateOutputRanges(
  outputValueRange: string,
  outputUncertaintyRange: string,
  variables: Variable[],
  spreadsheetAPI: SpreadsheetRef
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
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Invalid format';
    throw new Error(`Invalid output value range format: ${outputValueRange} - ${message}`);
  }

  try {
    normalizeRangeRef(outputUncertaintyRange);
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Invalid format';
    throw new Error(`Invalid output uncertainty range format: ${outputUncertaintyRange} - ${message}`);
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
    await spreadsheetAPI.getRange(outputValueRange);
  } catch (_error) {
    throw new Error(`Output value range "${outputValueRange}" is not accessible or out of bounds`);
  }

  try {
    await spreadsheetAPI.getRange(outputUncertaintyRange);
  } catch (_error) {
    throw new Error(`Output uncertainty range "${outputUncertaintyRange}" is not accessible or out of bounds`);
  }

  // 7. Check if ranges are writable (attempt to read and see if we get data)
  try {
    const testData = await spreadsheetAPI.getRange(outputValueRange);
    if (!Array.isArray(testData) || testData.length === 0) {
      throw new Error(`Output value range "${outputValueRange}" appears to be empty or inaccessible`);
    }
  } catch (_error) {
    throw new Error(`Output value range "${outputValueRange}" is not writable`);
  }

  try {
    const testData = await spreadsheetAPI.getRange(outputUncertaintyRange);
    if (!Array.isArray(testData) || testData.length === 0) {
      throw new Error(`Output uncertainty range "${outputUncertaintyRange}" appears to be empty or inaccessible`);
    }
  } catch (_error) {
    throw new Error(`Output uncertainty range "${outputUncertaintyRange}" is not writable`);
  }
}

/**
 * Validate the complete uncertainty propagation setup
 */
export async function validateUncertaintySetup(
  variables: Variable[],
  outputValueRange: string,
  outputUncertaintyRange: string,
  spreadsheetRef: SpreadsheetRef
): Promise<ValidationResult> {
  try {
    if (!spreadsheetRef.isReady()) {
      return { isValid: false, error: 'Spreadsheet not initialized' };
    }

    // Check basic requirements
    if (variables.some(v => !v.valueRange)) {
      return { isValid: false, error: 'All variables must have value ranges' };
    }

    if (!outputValueRange || !outputUncertaintyRange) {
      return { isValid: false, error: 'Output ranges are required' };
    }

    // Collect and validate input ranges
    const rangeRequests = collectRangeRequests(variables);
    if (rangeRequests.length === 0) {
      return { isValid: false, error: 'No input ranges to validate' };
    }

    // Read all input ranges in parallel
    const rangeResults = await Promise.all(
      rangeRequests.map(async (request) => {
        try {
          const data = await spreadsheetRef.getRange(request.range);
          return { ...request, data };
        } catch (error) {
          throw new Error(`Failed to read ${request.type} range "${request.range}" for variable "${request.variableName}": ${String(error)}`);
        }
      })
    );

    // Validate all input data
    validateRangeData(rangeResults, variables);

    // Validate output ranges
    await validateOutputRanges(outputValueRange, outputUncertaintyRange, variables, spreadsheetRef);

    return { isValid: true };
  } catch (error) {
    return { isValid: false, error: String(error) };
  }
}

/**
 * Execute uncertainty propagation
 */
export async function runUncertaintyPropagation(
  variables: Variable[],
  formula: string,
  outputValueRange: string,
  outputUncertaintyRange: string,
  outputConfidence: number,
  spreadsheetRef: SpreadsheetRef
): Promise<PropagationResult> {
  try {
    // Import the backend function
    const { invoke } = await import('@tauri-apps/api/core');

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
      return { success: false, error: result.error ?? 'Formula generation failed' };
    }

    // Parse output ranges to get starting cell
    const valueBounds = parseRange(outputValueRange);
    const uncBounds = parseRange(outputUncertaintyRange);

    if (!valueBounds || !uncBounds) {
      return { success: false, error: 'Invalid output range format' };
    }

    // Convert formulas to 2D arrays (column vectors)
    const valueFormulasArray = result.value_formulas.map(f => [{ f }]);
    const uncertaintyFormulasArray = result.uncertainty_formulas.map(f => [{ f }]);

    // Get starting cells (convert from 0-based to 1-based for A1 notation)
    const valueStartCell = `${columnToLetter(valueBounds.startCol)}${valueBounds.startRow + 1}`;
    const uncStartCell = `${columnToLetter(uncBounds.startCol)}${uncBounds.startRow + 1}`;

    // Write all value formulas at once
    await spreadsheetRef.updateRange(valueStartCell, valueFormulasArray);
    
    // Write all uncertainty formulas at once
    await spreadsheetRef.updateRange(uncStartCell, uncertaintyFormulasArray);

    return {
      success: true,
      valueFormulas: result.value_formulas,
      uncertaintyFormulas: result.uncertainty_formulas
    };
  } catch (error) {
    return { success: false, error: String(error) };
  }
}