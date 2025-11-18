// SidebarStateManager.ts - Centralized sidebar state management
import { useReducer, useCallback } from 'react';
import { ExportService } from '@/core/types/export';
import { ImportService } from '@/core/types/import';
import { SpreadsheetRef } from '@/tabs/spreadsheet/types/SpreadsheetInterface';

export type SidebarType = 'uncertainty' | 'unitConvert' | 'quickPlot' | 'export' | 'import' | 'statistics' | null;

export interface SidebarState {
  activeSidebar: SidebarType;
  unitConversion: {
    category: string;
    fromUnit: string;
    toUnit: string;
    value: string;
  };
  services: {
    exportService: ExportService | null;
    importService: ImportService | null;
  };
}

export type SidebarAction =
  | { type: 'SET_ACTIVE_SIDEBAR'; payload: SidebarType }
  | { type: 'SET_UNIT_CONVERSION_CATEGORY'; payload: string }
  | { type: 'SET_UNIT_CONVERSION_FROM_UNIT'; payload: string }
  | { type: 'SET_UNIT_CONVERSION_TO_UNIT'; payload: string }
  | { type: 'SET_UNIT_CONVERSION_VALUE'; payload: string }
  | { type: 'SET_EXPORT_SERVICE'; payload: ExportService }
  | { type: 'SET_IMPORT_SERVICE'; payload: ImportService }
  | { type: 'RESET_UNIT_CONVERSION' };

const initialState: SidebarState = {
  activeSidebar: null,
  unitConversion: {
    category: '',
    fromUnit: '',
    toUnit: '',
    value: '1',
  },
  services: {
    exportService: null,
    importService: null,
  },
};

function sidebarReducer(state: SidebarState, action: SidebarAction): SidebarState {
  switch (action.type) {
    case 'SET_ACTIVE_SIDEBAR':
      return { ...state, activeSidebar: action.payload };

    case 'SET_UNIT_CONVERSION_CATEGORY':
      return {
        ...state,
        unitConversion: { ...state.unitConversion, category: action.payload }
      };

    case 'SET_UNIT_CONVERSION_FROM_UNIT':
      return {
        ...state,
        unitConversion: { ...state.unitConversion, fromUnit: action.payload }
      };

    case 'SET_UNIT_CONVERSION_TO_UNIT':
      return {
        ...state,
        unitConversion: { ...state.unitConversion, toUnit: action.payload }
      };

    case 'SET_UNIT_CONVERSION_VALUE':
      return {
        ...state,
        unitConversion: { ...state.unitConversion, value: action.payload }
      };

    case 'SET_EXPORT_SERVICE':
      return {
        ...state,
        services: { ...state.services, exportService: action.payload }
      };

    case 'SET_IMPORT_SERVICE':
      return {
        ...state,
        services: { ...state.services, importService: action.payload }
      };

    case 'RESET_UNIT_CONVERSION':
      return {
        ...state,
        unitConversion: initialState.unitConversion
      };

    default:
      return state;
  }
}

/**
 * Custom hook for managing sidebar state with useReducer
 */
export function useSidebarState() {
  const [state, dispatch] = useReducer(sidebarReducer, initialState);

  const actions = {
    setActiveSidebar: useCallback((sidebar: SidebarType) => {
      dispatch({ type: 'SET_ACTIVE_SIDEBAR', payload: sidebar });
    }, []),

    setUnitConversionCategory: useCallback((category: string) => {
      dispatch({ type: 'SET_UNIT_CONVERSION_CATEGORY', payload: category });
    }, []),

    setUnitConversionFromUnit: useCallback((unit: string) => {
      dispatch({ type: 'SET_UNIT_CONVERSION_FROM_UNIT', payload: unit });
    }, []),

    setUnitConversionToUnit: useCallback((unit: string) => {
      dispatch({ type: 'SET_UNIT_CONVERSION_TO_UNIT', payload: unit });
    }, []),

    setUnitConversionValue: useCallback((value: string) => {
      dispatch({ type: 'SET_UNIT_CONVERSION_VALUE', payload: value });
    }, []),

    setExportService: useCallback((service: ExportService) => {
      dispatch({ type: 'SET_EXPORT_SERVICE', payload: service });
    }, []),

    setImportService: useCallback((service: ImportService) => {
      dispatch({ type: 'SET_IMPORT_SERVICE', payload: service });
    }, []),

    resetUnitConversion: useCallback(() => {
      dispatch({ type: 'RESET_UNIT_CONVERSION' });
    }, []),

    // Convenience method to initialize services
    initializeServices: useCallback((spreadsheetRef: React.RefObject<SpreadsheetRef | null>) => {
      if (spreadsheetRef.current) {
        const expSvc = spreadsheetRef.current.getExportService();
        const impSvc = spreadsheetRef.current.getImportService();
        dispatch({ type: 'SET_EXPORT_SERVICE', payload: expSvc });
        dispatch({ type: 'SET_IMPORT_SERVICE', payload: impSvc });
      }
    }, []),
  };

  return {
    state,
    actions,
  };
}