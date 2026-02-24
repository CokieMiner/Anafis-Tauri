/**
 * useUncertaintyPropagation hook - Extracted business logic for uncertainty propagation
 *
 * This hook encapsulates all the business logic for uncertainty propagation,
 * including state management, validation, and API calls. This reduces the
 * UncertaintySidebar component from 15 props to just 4 props.
 *
 * Supports both internal state (useState) and external state (from SidebarStateManager).
 */

import { useCallback, useState } from 'react';
import type { UncertaintyState } from '@/tabs/spreadsheet/managers/SidebarStateManager';
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
  // Optional external state (from SidebarStateManager)
  externalState?: UncertaintyState | undefined;
  externalActions?:
    | {
        setVariables: (variables: Variable[]) => void;
        addVariable: () => void;
        removeVariable: (index: number) => void;
        updateVariable: (
          index: number,
          field: keyof Variable,
          value: string | number
        ) => void;
        setFormula: (formula: string) => void;
        setOutputValueRange: (range: string) => void;
        setOutputUncertaintyRange: (range: string) => void;
        setOutputConfidence: (confidence: number) => void;
      }
    | undefined;
}

export function useUncertaintyPropagation({
  spreadsheetRef,
  onComplete,
  externalState,
  externalActions,
}: UseUncertaintyPropagationOptions) {
  // Internal state (used when external state is not provided)
  const [internalVariables, setInternalVariables] = useState<Variable[]>([
    {
      name: 'a',
      valueRange: 'A1:A10',
      uncertaintyRange: 'B1:B10',
      confidence: 95,
    },
  ]);
  const [internalFormula, setInternalFormula] = useState<string>('');
  const [internalOutputValueRange, setInternalOutputValueRange] =
    useState<string>('C1:C10');
  const [internalOutputUncertaintyRange, setInternalOutputUncertaintyRange] =
    useState<string>('D1:D10');
  const [internalOutputConfidence, setInternalOutputConfidence] =
    useState<number>(95);

  // Use external state if provided, otherwise use internal state
  const variables = externalState?.variables ?? internalVariables;
  const formula = externalState?.formula ?? internalFormula;
  const outputValueRange =
    externalState?.outputValueRange ?? internalOutputValueRange;
  const outputUncertaintyRange =
    externalState?.outputUncertaintyRange ?? internalOutputUncertaintyRange;
  const outputConfidence =
    externalState?.outputConfidence ?? internalOutputConfidence;

  // Processing state (always internal - transient UI state)
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

  // Determine which setters to use
  const setVariables = externalActions?.setVariables ?? setInternalVariables;
  const setFormula = externalActions?.setFormula ?? setInternalFormula;
  const setOutputValueRange =
    externalActions?.setOutputValueRange ?? setInternalOutputValueRange;
  const setOutputUncertaintyRange =
    externalActions?.setOutputUncertaintyRange ??
    setInternalOutputUncertaintyRange;
  const setOutputConfidence =
    externalActions?.setOutputConfidence ?? setInternalOutputConfidence;

  // Add a new variable
  const addVariable = useCallback(() => {
    if (externalActions?.addVariable) {
      externalActions.addVariable();
    } else {
      const nextName = generateNextVariableName(variables.length);
      const newVariable: Variable = {
        name: nextName,
        valueRange: '',
        uncertaintyRange: '',
        confidence: 95,
      };
      setVariables([...variables, newVariable]);
    }
  }, [externalActions, variables, generateNextVariableName, setVariables]);

  // Remove a variable
  const removeVariable = useCallback(
    (index: number) => {
      if (externalActions?.removeVariable) {
        externalActions.removeVariable(index);
      } else if (variables.length > 1) {
        setVariables(variables.filter((_, i) => i !== index));
      }
    },
    [externalActions, variables, setVariables]
  );

  // Update a variable
  const updateVariable = useCallback(
    (index: number, field: keyof Variable, value: string | number) => {
      if (externalActions?.updateVariable) {
        externalActions.updateVariable(index, field, value);
      } else {
        const updated = [...variables];
        const currentVar = updated[index];
        if (currentVar) {
          updated[index] = { ...currentVar, [field]: value } as Variable;
          setVariables(updated);
        }
      }
    },
    [externalActions, variables, setVariables]
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
      // Always validate against the latest spreadsheet contents.
      // This avoids stale validation results from short-lived cache entries.
      ValidationService.clearCache();

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
