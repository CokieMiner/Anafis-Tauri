/**
 * useUnitConversion hook - Extracted business logic for unit conversion operations
 *
 * This hook encapsulates all the business logic for unit conversions,
 * including state management, validation, and conversion calculations.
 */

import { useState, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface UseUnitConversionOptions {
  onSelectionChange?: (selection: string) => void;
}

interface UnitConversionResult {
  success: boolean;
  result?: number;
  formattedResult?: string;
  error?: string;
}

interface UnitInfo {
  symbol: string;
  name: string;
  category: string;
  description: string;
}

interface ConversionRequest {
  value: number;
  from_unit: string;
  to_unit: string;
}

interface ConversionResult {
  value: number;
  formatted_result: string;
  conversion_factor: number;
}

export function useUnitConversion({
  onSelectionChange,
}: UseUnitConversionOptions) {
  // Unit conversion state
  const [category, setCategory] = useState<string>('');
  const [fromUnit, setFromUnit] = useState<string>('');
  const [toUnit, setToUnit] = useState<string>('');
  const [value, setValue] = useState<string>('1');
  const [isConverting, setIsConverting] = useState<boolean>(false);
  const [lastResult, setLastResult] = useState<UnitConversionResult | null>(null);
  const [availableUnits, setAvailableUnits] = useState<Record<string, UnitInfo>>({});
  const [unitCategories, setUnitCategories] = useState<string[]>([]);

  // Load available units and categories on mount
  useEffect(() => {
    const loadUnits = async () => {
      try {
        const [units, categories] = await Promise.all([
          invoke<Record<string, UnitInfo>>('get_available_units'),
          invoke<string[]>('get_supported_categories')
        ]);
        setAvailableUnits(units);
        setUnitCategories(categories);
      } catch (error) {
        console.error('Failed to load unit data:', error);
      }
    };

    void loadUnits();
  }, []);

  // Get available units for selected category
  const getUnitsForCategory = useCallback((selectedCategory: string): string[] => {
    if (!selectedCategory) {
      return [];
    }

    return Object.values(availableUnits)
      .filter(unit => unit.category === selectedCategory)
      .map(unit => unit.symbol)
      .sort();
  }, [availableUnits]);

  // Perform unit conversion using Rust backend
  const convert = useCallback(async (): Promise<UnitConversionResult> => {
    try {
      const numValue = parseFloat(value);
      if (isNaN(numValue)) {
        return { success: false, error: 'Invalid numeric value' };
      }

      if (!category || !fromUnit || !toUnit) {
        return { success: false, error: 'Please select category and units' };
      }

      if (fromUnit === toUnit) {
        return {
          success: true,
          result: numValue,
          formattedResult: `${numValue} ${fromUnit} = ${numValue} ${toUnit}`
        };
      }

      // Use Rust backend for conversion
      const request: ConversionRequest = {
        value: numValue,
        from_unit: fromUnit,
        to_unit: toUnit,
      };

      const result = await invoke<ConversionResult>('convert_value', { request });

      return {
        success: true,
        result: result.value,
        formattedResult: result.formatted_result
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Conversion failed'
      };
    }
  }, [value, category, fromUnit, toUnit]);

  // Handle conversion with UI feedback
  const performConversion = useCallback(async (): Promise<void> => {
    setIsConverting(true);
    try {
      const result = await convert();
      setLastResult(result);
    } finally {
      setIsConverting(false);
    }
  }, [convert]);

  // Handle selection change
  const handleSelectionChange = useCallback((selection: string) => {
    onSelectionChange?.(selection);
  }, [onSelectionChange]);

  // Clear result
  const clearResult = useCallback(() => {
    setLastResult(null);
  }, []);

  return {
    // State
    category,
    fromUnit,
    toUnit,
    value,
    isConverting,
    lastResult,
    availableUnits: getUnitsForCategory(category),
    unitCategories,

    // Actions
    setCategory,
    setFromUnit,
    setToUnit,
    setValue,
    performConversion,
    handleSelectionChange,
    clearResult,
  };
}