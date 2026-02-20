// ValidationPipeline.ts - Unified validation pipeline for spreadsheet operations
import type { SpreadsheetRef } from '@/tabs/spreadsheet/types/SpreadsheetInterface';
import { RangeValidator } from './RangeValidator';

// Types for validation context and results
export interface ValidationContext {
  variables: Variable[];
  outputRanges: {
    value: string;
    uncertainty: string;
  };
  spreadsheet: SpreadsheetRef;
}

export interface Variable {
  name: string;
  valueRange?: string;
  uncertaintyRange?: string;
  confidence: number;
}

export interface ValidationResult {
  isValid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
}

export interface ValidationError {
  code: string;
  message: string;
  field?: string;
  value?: number | string;
}

export interface ValidationWarning {
  code: string;
  message: string;
  field?: string;
}

// Centralized error and warning definitions
const VALIDATION_ERRORS = {
  // Basic field validation
  MISSING_VARIABLES: {
    code: 'MISSING_VARIABLES',
    message:
      'At least one variable is required to perform uncertainty propagation.',
  },
  MISSING_VALUE_RANGE: {
    code: 'MISSING_VALUE_RANGE',
    message: 'Variable "{name}" requires a value range (e.g., "A1:A10").',
  },
  MISSING_OUTPUT_VALUE_RANGE: {
    code: 'MISSING_OUTPUT_VALUE_RANGE',
    message:
      'Output value range is required. Specify where to write the calculated values (e.g., "C1:C10").',
  },
  MISSING_OUTPUT_UNCERTAINTY_RANGE: {
    code: 'MISSING_OUTPUT_UNCERTAINTY_RANGE',
    message:
      'Output uncertainty range is required. Specify where to write the calculated uncertainties (e.g., "D1:D10").',
  },
  DUPLICATE_VARIABLE_NAMES: {
    code: 'DUPLICATE_VARIABLE_NAMES',
    message:
      'Duplicate variable names found: {duplicates}. Each variable must have a unique name.',
  },
  INVALID_CONFIDENCE_LEVEL: {
    code: 'INVALID_CONFIDENCE_LEVEL',
    message:
      'Variable "{name}" confidence level ({confidence}%) must be between 50% and 99.9%.',
  },

  // Range validation
  INVALID_RANGE_FORMAT: {
    code: 'INVALID_RANGE_FORMAT',
    message:
      'Range "{range}" is not valid A1 notation. Use format like "A1:A10" (column letter + row number).',
  },
  RANGES_OVERLAP: {
    code: 'RANGES_OVERLAP',
    message:
      'Output ranges cannot overlap. Value range "{valueRange}" and uncertainty range "{uncertaintyRange}" overlap. Choose different columns or adjust the ranges.',
  },
  RANGE_OVERLAPS_INPUT: {
    code: 'RANGE_OVERLAPS_INPUT',
    message:
      "Output ranges cannot overlap with input data ranges. Please choose different columns for your results that don't conflict with your input data.",
  },
  RANGE_NOT_ACCESSIBLE: {
    code: 'RANGE_NOT_ACCESSIBLE',
    message:
      'Range "{range}" is not accessible: {details}. Make sure the range exists and is not protected or read-only.',
  },
  RANGE_NOT_WRITABLE: {
    code: 'RANGE_NOT_WRITABLE',
    message:
      'Cannot write to range "{range}": {details}. Make sure the range is not protected or read-only.',
  },

  // Data validation
  NON_NUMERIC_DATA: {
    code: 'NON_NUMERIC_DATA',
    message:
      'Variable "{name}" {type} range "{range}" contains non-numeric data. All values must be numbers for uncertainty propagation.',
  },
  EMPTY_RANGE: {
    code: 'EMPTY_RANGE',
    message:
      'Variable "{name}" {type} range "{range}" is empty. Please provide numeric data.',
  },
  DIMENSION_MISMATCH: {
    code: 'DIMENSION_MISMATCH',
    message:
      'Variable "{name}": uncertainty range "{uncertaintyRange}" ({uncertaintyRows} rows) doesn\'t match value range "{valueRange}" ({valueRows} rows). Both ranges must have the same number of data points.',
  },

  // General validation
  SPREADSHEET_NOT_READY: {
    code: 'SPREADSHEET_NOT_READY',
    message:
      'Spreadsheet is not ready. Please wait for the spreadsheet to fully load before running uncertainty propagation.',
  },
  NO_INPUT_RANGES: {
    code: 'NO_INPUT_RANGES',
    message:
      'No input ranges found. Make sure at least one variable has a value range specified.',
  },
} as const;

const VALIDATION_WARNINGS = {
  NO_UNCERTAINTY_RANGES: {
    code: 'NO_UNCERTAINTY_RANGES',
    message:
      'Some variables have no uncertainty ranges - they will be treated as having zero uncertainty.',
  },
} as const;

// Validator interface
export interface Validator {
  validate(context: ValidationContext): Promise<ValidationResult>;
}

// Basic field validation (names, ranges, confidence levels)
class BasicFieldValidator implements Validator {
  validate(context: ValidationContext): Promise<ValidationResult> {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    const { variables, outputRanges } = context;

    // Check for variables
    if (variables.length === 0) {
      errors.push(VALIDATION_ERRORS.MISSING_VARIABLES);
    }

    // Check for missing value ranges
    const missingValueRanges = variables.filter((v) => !v.valueRange?.trim());
    if (missingValueRanges.length > 0) {
      const names = missingValueRanges.map((v) => `"${v.name}"`).join(', ');
      errors.push({
        ...VALIDATION_ERRORS.MISSING_VALUE_RANGE,
        message: VALIDATION_ERRORS.MISSING_VALUE_RANGE.message.replace(
          '{name}',
          names
        ),
      });
    }

    // Check output ranges
    if (!outputRanges.value.trim()) {
      errors.push(VALIDATION_ERRORS.MISSING_OUTPUT_VALUE_RANGE);
    }
    if (!outputRanges.uncertainty.trim()) {
      errors.push(VALIDATION_ERRORS.MISSING_OUTPUT_UNCERTAINTY_RANGE);
    }

    // Check for duplicate variable names
    const names = variables.map((v) => v.name.trim()).filter(Boolean);
    const duplicates = names.filter(
      (name, index) => names.indexOf(name) !== index
    );
    if (duplicates.length > 0) {
      errors.push({
        ...VALIDATION_ERRORS.DUPLICATE_VARIABLE_NAMES,
        message: VALIDATION_ERRORS.DUPLICATE_VARIABLE_NAMES.message.replace(
          '{duplicates}',
          duplicates.join(', ')
        ),
      });
    }

    // Check confidence levels
    variables.forEach((v) => {
      if (v.confidence < 50 || v.confidence > 99.9) {
        errors.push({
          ...VALIDATION_ERRORS.INVALID_CONFIDENCE_LEVEL,
          message: VALIDATION_ERRORS.INVALID_CONFIDENCE_LEVEL.message
            .replace('{name}', v.name)
            .replace('{confidence}', v.confidence.toString()),
          field: 'confidence',
          value: v.confidence,
        });
      }
    });

    // Check for missing uncertainty ranges (warning only)
    if (variables.some((v) => !v.uncertaintyRange?.trim())) {
      warnings.push(VALIDATION_WARNINGS.NO_UNCERTAINTY_RANGES);
    }

    return Promise.resolve({
      isValid: errors.length === 0,
      errors,
      warnings,
    });
  }
}

// Range format and structure validation
class RangeFormatValidator implements Validator {
  validate(context: ValidationContext): Promise<ValidationResult> {
    const errors: ValidationError[] = [];
    const { outputRanges } = context;

    // Validate output range formats
    try {
      RangeValidator.validateFormat(outputRanges.value);
    } catch (_error) {
      errors.push({
        ...VALIDATION_ERRORS.INVALID_RANGE_FORMAT,
        message: VALIDATION_ERRORS.INVALID_RANGE_FORMAT.message.replace(
          '{range}',
          outputRanges.value
        ),
      });
    }

    try {
      RangeValidator.validateFormat(outputRanges.uncertainty);
    } catch (_error) {
      errors.push({
        ...VALIDATION_ERRORS.INVALID_RANGE_FORMAT,
        message: VALIDATION_ERRORS.INVALID_RANGE_FORMAT.message.replace(
          '{range}',
          outputRanges.uncertainty
        ),
      });
    }

    return Promise.resolve({
      isValid: errors.length === 0,
      errors,
      warnings: [],
    });
  }
}

// Range overlap validation
class OverlapValidator implements Validator {
  validate(context: ValidationContext): Promise<ValidationResult> {
    const errors: ValidationError[] = [];
    const { variables, outputRanges } = context;

    // Check output ranges don't overlap with each other
    try {
      RangeValidator.validateNoOverlap(
        outputRanges.value,
        outputRanges.uncertainty
      );
    } catch (_error) {
      errors.push({
        ...VALIDATION_ERRORS.RANGES_OVERLAP,
        message: VALIDATION_ERRORS.RANGES_OVERLAP.message
          .replace('{valueRange}', outputRanges.value)
          .replace('{uncertaintyRange}', outputRanges.uncertainty),
      });
    }

    // Check output ranges don't overlap with input ranges
    const inputRanges: string[] = [];
    for (const variable of variables) {
      if (variable.valueRange) {
        inputRanges.push(variable.valueRange);
      }
      if (variable.uncertaintyRange) {
        inputRanges.push(variable.uncertaintyRange);
      }
    }

    try {
      RangeValidator.validateNoOverlapWithReferences(
        [outputRanges.value, outputRanges.uncertainty],
        inputRanges
      );
    } catch (_error) {
      errors.push(VALIDATION_ERRORS.RANGE_OVERLAPS_INPUT);
    }

    return Promise.resolve({
      isValid: errors.length === 0,
      errors,
      warnings: [],
    });
  }
}

// Spreadsheet accessibility validation
class AccessibilityValidator implements Validator {
  async validate(context: ValidationContext): Promise<ValidationResult> {
    const errors: ValidationError[] = [];
    const { outputRanges, spreadsheet } = context;

    // Check spreadsheet readiness
    if (!spreadsheet.isReady()) {
      errors.push(VALIDATION_ERRORS.SPREADSHEET_NOT_READY);
      return { isValid: false, errors, warnings: [] };
    }

    // Validate range accessibility and writability in parallel
    const validationPromises = [
      RangeValidator.validateAccessible(outputRanges.value, spreadsheet).catch(
        (_error: unknown) => ({
          type: 'accessible' as const,
          range: outputRanges.value,
          error: String(_error),
        })
      ),
      RangeValidator.validateAccessible(
        outputRanges.uncertainty,
        spreadsheet
      ).catch((_error: unknown) => ({
        type: 'accessible' as const,
        range: outputRanges.uncertainty,
        error: String(_error),
      })),
      RangeValidator.validateWritable(outputRanges.value, spreadsheet).catch(
        (_error: unknown) => ({
          type: 'writable' as const,
          range: outputRanges.value,
          error: String(_error),
        })
      ),
      RangeValidator.validateWritable(
        outputRanges.uncertainty,
        spreadsheet
      ).catch((_error: unknown) => ({
        type: 'writable' as const,
        range: outputRanges.uncertainty,
        error: String(_error),
      })),
    ];

    const results = await Promise.allSettled(validationPromises);

    for (const result of results) {
      if (result.status === 'rejected' && result.reason) {
        const reason = result.reason as {
          type: 'accessible' | 'writable';
          range: string;
          error: string;
        };
        const { type, range, error } = reason;
        if (type === 'accessible') {
          errors.push({
            ...VALIDATION_ERRORS.RANGE_NOT_ACCESSIBLE,
            message: VALIDATION_ERRORS.RANGE_NOT_ACCESSIBLE.message
              .replace('{range}', range)
              .replace('{details}', error),
          });
        } else {
          errors.push({
            ...VALIDATION_ERRORS.RANGE_NOT_WRITABLE,
            message: VALIDATION_ERRORS.RANGE_NOT_WRITABLE.message
              .replace('{range}', range)
              .replace('{details}', error),
          });
        }
      }
    }

    return {
      isValid: errors.length === 0,
      errors,
      warnings: [],
    };
  }
}

// Input data validation
class DataValidator implements Validator {
  async validate(context: ValidationContext): Promise<ValidationResult> {
    const errors: ValidationError[] = [];
    const { variables, spreadsheet } = context;

    // Collect all range requests
    const rangeRequests: Array<{
      variable: Variable;
      type: 'value' | 'uncertainty';
      range: string;
    }> = [];

    for (const variable of variables) {
      if (variable.valueRange) {
        rangeRequests.push({
          variable,
          type: 'value',
          range: variable.valueRange,
        });
      }
      if (variable.uncertaintyRange) {
        rangeRequests.push({
          variable,
          type: 'uncertainty',
          range: variable.uncertaintyRange,
        });
      }
    }

    if (rangeRequests.length === 0) {
      errors.push(VALIDATION_ERRORS.NO_INPUT_RANGES);
      return { isValid: false, errors, warnings: [] };
    }

    // Read all ranges in parallel
    const successfulResults: Array<{
      variable: Variable;
      type: 'value' | 'uncertainty';
      range: string;
      data: (string | number | null)[][];
    }> = [];

    const failedResults: Array<{
      variable: Variable;
      type: 'value' | 'uncertainty';
      range: string;
      error: string;
    }> = [];

    await Promise.all(
      rangeRequests.map(async (request) => {
        try {
          const data = await spreadsheet.getRange(request.range);
          successfulResults.push({ ...request, data });
        } catch (_error: unknown) {
          failedResults.push({ ...request, error: String(_error) });
        }
      })
    );

    // Add errors for failed reads
    for (const failed of failedResults) {
      errors.push({
        code: 'RANGE_READ_ERROR',
        message: `Cannot read ${failed.type} range "${failed.range}" for variable "${failed.variable.name}": ${failed.error}`,
      });
    }

    // Group successful results by variable
    const resultsByVariable = new Map<
      string,
      {
        value?: { data: (string | number | null)[][]; range: string };
        uncertainty?: { data: (string | number | null)[][]; range: string };
      }
    >();

    for (const result of successfulResults) {
      const key = result.variable.name;
      if (!resultsByVariable.has(key)) {
        resultsByVariable.set(key, {});
      }

      const varResults = resultsByVariable.get(key);
      if (!varResults) continue;
      if (result.type === 'value') {
        varResults.value = { data: result.data, range: result.range };
      } else {
        varResults.uncertainty = { data: result.data, range: result.range };
      }
    }

    // Validate data for each variable
    for (const [varName, results] of resultsByVariable) {
      const variable = variables.find((v) => v.name === varName);
      if (!variable) continue;

      // Validate value data
      if (results.value) {
        this.validateNumericData(
          variable,
          'value',
          results.value.data,
          results.value.range,
          errors
        );
      }

      // Validate uncertainty data and dimension matching
      if (results.uncertainty && results.value) {
        this.validateNumericData(
          variable,
          'uncertainty',
          results.uncertainty.data,
          results.uncertainty.range,
          errors
        );
        this.validateDimensions(
          variable,
          results.value,
          results.uncertainty,
          errors
        );
      }
    }

    return {
      isValid: errors.length === 0,
      errors,
      warnings: [],
    };
  }

  private validateNumericData(
    variable: Variable,
    type: 'value' | 'uncertainty',
    data: (string | number | null)[][],
    range: string,
    errors: ValidationError[]
  ): void {
    // Check if all data is numeric
    const isAllNumeric = data.every((row) =>
      row.every((cell) => typeof cell === 'number' && Number.isFinite(cell))
    );

    if (!isAllNumeric) {
      errors.push({
        ...VALIDATION_ERRORS.NON_NUMERIC_DATA,
        message: VALIDATION_ERRORS.NON_NUMERIC_DATA.message
          .replace('{name}', variable.name)
          .replace('{type}', type)
          .replace('{range}', range),
      });
    }

    // Check if range is empty
    if (data.length === 0 || data[0]?.length === 0) {
      errors.push({
        ...VALIDATION_ERRORS.EMPTY_RANGE,
        message: VALIDATION_ERRORS.EMPTY_RANGE.message
          .replace('{name}', variable.name)
          .replace('{type}', type)
          .replace('{range}', range),
      });
    }
  }

  private validateDimensions(
    variable: Variable,
    valueResult: { data: (string | number | null)[][]; range: string },
    uncertaintyResult: { data: (string | number | null)[][]; range: string },
    errors: ValidationError[]
  ): void {
    if (valueResult.data.length !== uncertaintyResult.data.length) {
      errors.push({
        ...VALIDATION_ERRORS.DIMENSION_MISMATCH,
        message: VALIDATION_ERRORS.DIMENSION_MISMATCH.message
          .replace('{name}', variable.name)
          .replace('{valueRange}', valueResult.range)
          .replace('{uncertaintyRange}', uncertaintyResult.range)
          .replace('{valueRows}', valueResult.data.length.toString())
          .replace(
            '{uncertaintyRows}',
            uncertaintyResult.data.length.toString()
          ),
      });
    }
  }
}

// Main validation pipeline
let activeValidators: Validator[] = [
  new BasicFieldValidator(),
  new RangeFormatValidator(),
  new OverlapValidator(),
  new AccessibilityValidator(),
  new DataValidator(),
];

export const ValidationPipeline = {
  /**
   * Run the complete validation pipeline for uncertainty propagation setup
   */
  async validateUncertaintySetup(
    variables: Variable[],
    outputValueRange: string,
    outputUncertaintyRange: string,
    spreadsheet: SpreadsheetRef
  ): Promise<ValidationResult> {
    const context: ValidationContext = {
      variables,
      outputRanges: {
        value: outputValueRange,
        uncertainty: outputUncertaintyRange,
      },
      spreadsheet,
    };

    // Run all validators and collect results
    const allErrors: ValidationError[] = [];
    const allWarnings: ValidationWarning[] = [];

    for (const validator of activeValidators) {
      try {
        const result = await validator.validate(context);

        // If a validator fails completely, we can stop early for performance
        if (!result.isValid && result.errors.length > 0) {
          // For critical failures (like missing fields), stop the pipeline
          if (
            result.errors.some((e) =>
              [
                'MISSING_VARIABLES',
                'MISSING_VALUE_RANGE',
                'MISSING_OUTPUT_VALUE_RANGE',
                'MISSING_OUTPUT_UNCERTAINTY_RANGE',
              ].includes(e.code)
            )
          ) {
            return {
              isValid: false,
              errors: [...allErrors, ...result.errors],
              warnings: [...allWarnings, ...result.warnings],
            };
          }
        }

        allErrors.push(...result.errors);
        allWarnings.push(...result.warnings);
      } catch (error) {
        // If a validator throws an unexpected error, add it as a validation error
        allErrors.push({
          code: 'VALIDATOR_ERROR',
          message: `Validation failed unexpectedly: ${String(error)}`,
        });
      }
    }

    return {
      isValid: allErrors.length === 0,
      errors: allErrors,
      warnings: allWarnings,
    };
  },

  /**
   * Add a custom validator to the pipeline (for extensibility)
   */
  addValidator(validator: Validator): void {
    activeValidators.push(validator);
  },

  /**
   * Replace the default validators (for testing or customization)
   */
  setValidators(validators: Validator[]): void {
    activeValidators = [...validators];
  },
};
