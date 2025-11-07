// uncertaintyOperations.ts - High-level uncertainty propagation operations
import { SpreadsheetRef } from '@/tabs/spreadsheet/types/SpreadsheetInterface';
import { parseRange, columnToLetter } from '@/tabs/spreadsheet/univer/utils/cellUtils';
import { RangeValidator } from '@/tabs/spreadsheet/univer/utils/RangeValidator';
import { Result, ok, err } from '@/core/types/result';

// Types for uncertainty operations
export interface Variable {
  name: string;
  valueRange?: string;
  uncertaintyRange?: string;
  confidence: number;
}

export interface ValidationError {
  message: string;
  code?: string;
}

export interface PropagationResult {
  valueFormulas?: string[];
  uncertaintyFormulas?: string[];
}

export interface PropagationError {
  message: string;
  code?: string;
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
 * Validate output ranges comprehensively using RangeValidator
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

  RangeValidator.validateFormat(outputValueRange);
  RangeValidator.validateFormat(outputUncertaintyRange);

  // 2. Check for self-intersection between output ranges
  RangeValidator.validateNoOverlap(outputValueRange, outputUncertaintyRange);

  // 3. Collect all input ranges for overlap checking
  const inputRanges: string[] = [];
  for (const variable of variables) {
    if (variable.valueRange) {
      inputRanges.push(variable.valueRange);
    }
    if (variable.uncertaintyRange) {
      inputRanges.push(variable.uncertaintyRange);
    }
  }

  // 4. Check output ranges don't overlap with input ranges
  RangeValidator.validateNoOverlapWithReferences([outputValueRange, outputUncertaintyRange], inputRanges);

  // 5. Verify ranges exist and are accessible
  await RangeValidator.validateAccessible(outputValueRange, spreadsheetAPI);
  await RangeValidator.validateAccessible(outputUncertaintyRange, spreadsheetAPI);

  // 6. Check if ranges are writable
  await RangeValidator.validateWritable(outputValueRange, spreadsheetAPI);
  await RangeValidator.validateWritable(outputUncertaintyRange, spreadsheetAPI);
}

/**
 * Validate the complete uncertainty propagation setup
 */
export async function validateUncertaintySetup(
  variables: Variable[],
  outputValueRange: string,
  outputUncertaintyRange: string,
  spreadsheetRef: SpreadsheetRef
): Promise<Result<void, ValidationError>> {
  try {
    if (!spreadsheetRef.isReady()) {
      return err({ message: 'Spreadsheet not initialized', code: 'SPREADSHEET_NOT_READY' });
    }

    // Check basic requirements
    if (variables.some(v => !v.valueRange)) {
      return err({ message: 'All variables must have value ranges', code: 'MISSING_VALUE_RANGES' });
    }

    if (!outputValueRange || !outputUncertaintyRange) {
      return err({ message: 'Output ranges are required', code: 'MISSING_OUTPUT_RANGES' });
    }

    // Collect and validate input ranges
    const rangeRequests = collectRangeRequests(variables);
    if (rangeRequests.length === 0) {
      return err({ message: 'No input ranges to validate', code: 'NO_INPUT_RANGES' });
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

    return ok(undefined);
  } catch (error) {
    return err({ message: String(error), code: 'VALIDATION_FAILED' });
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
): Promise<Result<PropagationResult, PropagationError>> {
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
      return err({ message: result.error ?? 'Formula generation failed', code: 'FORMULA_GENERATION_FAILED' });
    }

    // Parse output ranges to get starting cell
    const valueBounds = parseRange(outputValueRange);
    const uncBounds = parseRange(outputUncertaintyRange);

    if (!valueBounds || !uncBounds) {
      return err({ message: 'Invalid output range format', code: 'INVALID_RANGE_FORMAT' });
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

    return ok({
      valueFormulas: result.value_formulas,
      uncertaintyFormulas: result.uncertainty_formulas
    });
  } catch (error) {
    return err({ message: String(error), code: 'PROPAGATION_FAILED' });
  }
}