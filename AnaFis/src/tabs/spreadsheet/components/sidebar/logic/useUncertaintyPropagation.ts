/**
 * useUncertaintyPropagation hook - Extracted business logic for uncertainty propagation
 *
 * This hook encapsulates all the business logic for uncertainty propagation,
 * including state management, validation, and API calls. This reduces the
 * UncertaintySidebar component from 15 props to just 4 props.
 */

import { useCallback, useState } from 'react';
import type { SpreadsheetRef } from '@/tabs/spreadsheet/types/SpreadsheetInterface';
import {
  runUncertaintyPropagation,
  type Variable,
} from '@/tabs/spreadsheet/univer/operations/uncertaintyOperations';
import {
  type ValidationResult,
  ValidationService,
} from '@/tabs/spreadsheet/univer/utils/ValidationService';

interface UseUncertaintyPropagationOptions {
  spreadsheetRef: React.RefObject<SpreadsheetRef | null>;
  onComplete?: (resultRange: string) => void;
}

export function useUncertaintyPropagation({
  spreadsheetRef,
  onComplete,
}: UseUncertaintyPropagationOptions) {
  // All state is now managed in the hook
  const [variables, setVariables] = useState<Variable[]>([
    {
      name: 'a',
      valueRange: 'A1:A10',
      uncertaintyRange: 'B1:B10',
      confidence: 95,
    },
  ]);
  const [formula, setFormula] = useState<string>('');
  const [outputValueRange, setOutputValueRange] = useState<string>('C1:C10');
  const [outputUncertaintyRange, setOutputUncertaintyRange] =
    useState<string>('D1:D10');
  const [outputConfidence, setOutputConfidence] = useState<number>(95);
  const [isProcessing, setIsProcessing] = useState<boolean>(false);
  const [error, setError] = useState<string>('');

  // Generate next variable name: a-z, then aa-zz
  const generateNextVariableName = useCallback(
    (variableCount: number): string => {
      if (variableCount < 26) {
        // a-z
        return String.fromCharCode(97 + variableCount);
      } else {
        // aa-zz
        const doubleIndex = variableCount - 26;
        const firstChar = String.fromCharCode(
          97 + Math.floor(doubleIndex / 26)
        );
        const secondChar = String.fromCharCode(97 + (doubleIndex % 26));
        return firstChar + secondChar;
      }
    },
    []
  );

  // Add a new variable
  const addVariable = useCallback(() => {
    const nextName = generateNextVariableName(variables.length);
    const newVariable: Variable = {
      name: nextName,
      valueRange: '',
      uncertaintyRange: '',
      confidence: 95,
    };
    setVariables([...variables, newVariable]);
  }, [variables, generateNextVariableName]);

  // Remove a variable
  const removeVariable = useCallback(
    (index: number) => {
      if (variables.length > 1) {
        setVariables(variables.filter((_, i) => i !== index));
      }
    },
    [variables]
  );

  // Update a variable
  const updateVariable = useCallback(
    (index: number, field: keyof Variable, value: string | number) => {
      const updated = [...variables];
      const currentVar = updated[index];
      if (currentVar) {
        updated[index] = { ...currentVar, [field]: value } as Variable;
        setVariables(updated);
      }
    },
    [variables]
  );

  // Validate the current setup using consolidated validation service
  const validateSetup = useCallback(async (): Promise<ValidationResult> => {
    const spreadsheetAPI = spreadsheetRef.current;
    if (!spreadsheetAPI) {
      return {
        isValid: false,
        errors: ['Spreadsheet not initialized'],
        warnings: [],
      };
    }

    return await ValidationService.validateUncertaintySetup(
      variables,
      outputValueRange,
      outputUncertaintyRange,
      spreadsheetAPI
    );
  }, [variables, spreadsheetRef, outputValueRange, outputUncertaintyRange]);

  // Execute uncertainty propagation
  const propagate = useCallback(async () => {
    setError('');
    setIsProcessing(true);

    // Basic validation
    if (variables.some((v) => !v.valueRange)) {
      setError('Fill in all value ranges');
      setIsProcessing(false);
      return;
    }
    if (!formula || !outputValueRange || !outputUncertaintyRange) {
      setError('Fill in formula and output ranges');
      setIsProcessing(false);
      return;
    }

    if (!spreadsheetRef.current) {
      setError('Spreadsheet not initialized');
      setIsProcessing(false);
      return;
    }

    try {
      // Validate data before sending to backend
      const validationResult = await validateSetup();
      if (!validationResult.isValid) {
        setError(validationResult.errors.join('; '));
        setIsProcessing(false);
        return;
      }

      // Show warnings if any
      if (validationResult.warnings.length > 0) {
        console.warn('Validation warnings:', validationResult.warnings);
      }

      // Run uncertainty propagation
      const result = await runUncertaintyPropagation(
        variables,
        formula,
        outputValueRange,
        outputUncertaintyRange,
        outputConfidence,
        spreadsheetRef.current
      );

      if (!result.ok) {
        setError(result.error.message);
        return;
      }

      onComplete?.(outputValueRange);
      setError('');
    } catch (err: unknown) {
      console.error('Propagation error:', err);
      setError(String(err));
    } finally {
      setIsProcessing(false);
    }
  }, [
    variables,
    formula,
    outputValueRange,
    outputUncertaintyRange,
    outputConfidence,
    spreadsheetRef,
    validateSetup,
    onComplete,
  ]);

  // Get current variable names for formula hints
  const variableNames = variables.map((v) => v.name);

  return {
    // State
    variables,
    formula,
    outputValueRange,
    outputUncertaintyRange,
    outputConfidence,
    isProcessing,
    error,
    variableNames,

    // Actions
    setVariables,
    setFormula,
    setOutputValueRange,
    setOutputUncertaintyRange,
    setOutputConfidence,
    addVariable,
    removeVariable,
    updateVariable,
    propagate,
    clearError: () => setError(''),
  };
}
