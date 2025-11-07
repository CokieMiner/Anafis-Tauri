import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface UnitInfo {
  symbol: string;
  name: string;
  category: string;
  description: string;
}

export interface ConversionResult {
  value: number;
  formatted_result: string;
  conversion_factor: number;
}

// Cache for unit metadata (shared across all instances)
const unitMetadataCache = new Map<string, UnitInfo>();
let cacheInitialized = false;

/**
 * Custom hook for managing unit data (categories and available units)
 */
export const useUnitData = () => {
  const [categories, setCategories] = useState<string[]>([]);
  const [availableUnits, setAvailableUnits] = useState<Record<string, UnitInfo>>({});
  const [isLoadingCategories, setIsLoadingCategories] = useState(false);
  const [isLoadingUnits, setIsLoadingUnits] = useState(false);
  const [error, setError] = useState<string>('');

  // Load categories
  const loadCategories = useCallback(async () => {
    try {
      setIsLoadingCategories(true);
      setError('');
      const cats: string[] = await invoke('get_supported_categories');
      // Add "All" category at the beginning
      const allCategories = ['All', ...cats];
      setCategories(allCategories);
    } catch (err) {
      setError('Failed to load categories');
      console.error(err);
    } finally {
      setIsLoadingCategories(false);
    }
  }, []);

  // Load units
  const loadUnits = useCallback(async () => {
    try {
      setIsLoadingUnits(true);
      setError('');

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
    } finally {
      setIsLoadingUnits(false);
    }
  }, []);

  // Filter units by category and search query
  const getFilteredUnits = useCallback((category: string, searchQuery: string) => {
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
  }, [availableUnits]);

  // Check unit compatibility
  const checkUnitCompatibility = useCallback(async (fromUnit: string, toUnit: string): Promise<boolean> => {
    try {
      const compatible: boolean = await invoke('check_unit_compatibility', {
        fromUnit,
        toUnit
      });
      return compatible;
    } catch (err) {
      console.error('Error checking compatibility:', err);
      return false;
    }
  }, []);

  return {
    categories,
    availableUnits,
    isLoadingCategories,
    isLoadingUnits,
    error,
    loadCategories,
    loadUnits,
    getFilteredUnits,
    checkUnitCompatibility
  };
};