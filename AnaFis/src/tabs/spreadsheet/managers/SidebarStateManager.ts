// SidebarStateManager.ts - Centralized sidebar state management
import { useCallback, useReducer } from 'react';
import type { ExportService } from '@/core/types/export';
import type { ImportService } from '@/core/types/import';
import type { SpreadsheetRef } from '@/tabs/spreadsheet/types/SpreadsheetInterface';
import type { Variable } from '@/tabs/spreadsheet/univer/operations/uncertaintyOperations';

export type SidebarType =
  | 'uncertainty'
  | 'unitConvert'
  | 'quickPlot'
  | 'export'
  | 'import'
  | 'statistics'
  | null;

// Quick Plot state types
export type PlotType = 'scatter' | 'line' | 'both';
export type ExportTheme = 'dark' | 'light';
export type QuickPlotExportFormat = 'png' | 'svg';

// Export sidebar state types
export type ExportFormat =
  | 'anafispread'
  | 'csv'
  | 'txt'
  | 'json'
  | 'xlsx'
  | 'parquet';
export type ExportRangeMode = 'sheet' | 'selection' | 'custom';
export type ExportMode = 'file' | 'library';

// Import sidebar state types
export type ImportMode = 'file' | 'library';

// Quick Plot state interface
export interface QuickPlotState {
  xRange: string;
  yRange: string;
  errorRange: string;
  xLabel: string;
  yLabel: string;
  plotType: PlotType;
  showErrorBars: boolean;
  exportTheme: ExportTheme;
  exportFormat: QuickPlotExportFormat;
}

// Uncertainty state interface
export interface UncertaintyState {
  variables: Variable[];
  formula: string;
  outputValueRange: string;
  outputUncertaintyRange: string;
  outputConfidence: number;
}

// Export sidebar state interface
export interface ExportSidebarState {
  format: ExportFormat;
  rangeMode: ExportRangeMode;
  customRange: string;
  customDelimiter: string;
  mode: ExportMode;
  // Library export state
  libraryName: string;
  libraryDescription: string;
  libraryTags: string;
  libraryUnit: string;
  dataRange: string;
  uncertaintyRange: string;
}

// Import sidebar state interface
export interface ImportSidebarState {
  mode: ImportMode;
  targetRange: string;
  libraryDataRange: string;
  libraryUncertaintyRange: string;
}

export interface SidebarState {
  activeSidebar: SidebarType;
  unitConversion: {
    category: string;
    fromUnit: string;
    toUnit: string;
    value: string;
  };
  quickPlot: QuickPlotState;
  uncertainty: UncertaintyState;
  export: ExportSidebarState;
  import: ImportSidebarState;
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
  | { type: 'RESET_UNIT_CONVERSION' }
  // Quick Plot actions
  | { type: 'SET_QUICK_PLOT_X_RANGE'; payload: string }
  | { type: 'SET_QUICK_PLOT_Y_RANGE'; payload: string }
  | { type: 'SET_QUICK_PLOT_ERROR_RANGE'; payload: string }
  | { type: 'SET_QUICK_PLOT_X_LABEL'; payload: string }
  | { type: 'SET_QUICK_PLOT_Y_LABEL'; payload: string }
  | { type: 'SET_QUICK_PLOT_TYPE'; payload: PlotType }
  | { type: 'SET_QUICK_PLOT_SHOW_ERROR_BARS'; payload: boolean }
  | { type: 'SET_QUICK_PLOT_EXPORT_THEME'; payload: ExportTheme }
  | { type: 'SET_QUICK_PLOT_EXPORT_FORMAT'; payload: QuickPlotExportFormat }
  | { type: 'RESET_QUICK_PLOT' }
  // Uncertainty actions
  | { type: 'SET_UNCERTAINTY_VARIABLES'; payload: Variable[] }
  | { type: 'ADD_UNCERTAINTY_VARIABLE'; payload: Variable }
  | { type: 'REMOVE_UNCERTAINTY_VARIABLE'; payload: number }
  | {
      type: 'UPDATE_UNCERTAINTY_VARIABLE';
      payload: { index: number; field: keyof Variable; value: string | number };
    }
  | { type: 'SET_UNCERTAINTY_FORMULA'; payload: string }
  | { type: 'SET_UNCERTAINTY_OUTPUT_VALUE_RANGE'; payload: string }
  | { type: 'SET_UNCERTAINTY_OUTPUT_UNCERTAINTY_RANGE'; payload: string }
  | { type: 'SET_UNCERTAINTY_OUTPUT_CONFIDENCE'; payload: number }
  | { type: 'RESET_UNCERTAINTY' }
  // Export sidebar actions
  | { type: 'SET_EXPORT_FORMAT'; payload: ExportFormat }
  | { type: 'SET_EXPORT_RANGE_MODE'; payload: ExportRangeMode }
  | { type: 'SET_EXPORT_CUSTOM_RANGE'; payload: string }
  | { type: 'SET_EXPORT_CUSTOM_DELIMITER'; payload: string }
  | { type: 'SET_EXPORT_MODE'; payload: ExportMode }
  | { type: 'SET_EXPORT_LIBRARY_NAME'; payload: string }
  | { type: 'SET_EXPORT_LIBRARY_DESCRIPTION'; payload: string }
  | { type: 'SET_EXPORT_LIBRARY_TAGS'; payload: string }
  | { type: 'SET_EXPORT_LIBRARY_UNIT'; payload: string }
  | { type: 'SET_EXPORT_DATA_RANGE'; payload: string }
  | { type: 'SET_EXPORT_UNCERTAINTY_RANGE'; payload: string }
  | { type: 'RESET_EXPORT' }
  // Import sidebar actions
  | { type: 'SET_IMPORT_MODE'; payload: ImportMode }
  | { type: 'SET_IMPORT_TARGET_RANGE'; payload: string }
  | { type: 'SET_IMPORT_LIBRARY_DATA_RANGE'; payload: string }
  | { type: 'SET_IMPORT_LIBRARY_UNCERTAINTY_RANGE'; payload: string }
  | { type: 'RESET_IMPORT' };

const initialQuickPlotState: QuickPlotState = {
  xRange: '',
  yRange: '',
  errorRange: '',
  xLabel: '',
  yLabel: '',
  plotType: 'scatter',
  showErrorBars: false,
  exportTheme: 'dark',
  exportFormat: 'png',
};

const initialUncertaintyState: UncertaintyState = {
  variables: [
    {
      name: 'a',
      valueRange: 'A1:A10',
      uncertaintyRange: 'B1:B10',
      confidence: 95,
    },
  ],
  formula: '',
  outputValueRange: 'C1:C10',
  outputUncertaintyRange: 'D1:D10',
  outputConfidence: 95,
};

const initialExportState: ExportSidebarState = {
  format: 'anafispread',
  rangeMode: 'sheet',
  customRange: '',
  customDelimiter: '|',
  mode: 'file',
  libraryName: '',
  libraryDescription: '',
  libraryTags: '',
  libraryUnit: '',
  dataRange: 'A:A',
  uncertaintyRange: '',
};

const initialImportState: ImportSidebarState = {
  mode: 'file',
  targetRange: 'A1',
  libraryDataRange: 'A1',
  libraryUncertaintyRange: 'B1',
};

const initialState: SidebarState = {
  activeSidebar: null,
  unitConversion: {
    category: '',
    fromUnit: '',
    toUnit: '',
    value: '1',
  },
  quickPlot: initialQuickPlotState,
  uncertainty: initialUncertaintyState,
  export: initialExportState,
  import: initialImportState,
  services: {
    exportService: null,
    importService: null,
  },
};

// Helper function to generate next variable name
function generateNextVariableName(variableCount: number): string {
  if (variableCount < 26) {
    return String.fromCharCode(97 + variableCount);
  } else {
    const doubleIndex = variableCount - 26;
    const firstChar = String.fromCharCode(97 + Math.floor(doubleIndex / 26));
    const secondChar = String.fromCharCode(97 + (doubleIndex % 26));
    return firstChar + secondChar;
  }
}

function sidebarReducer(
  state: SidebarState,
  action: SidebarAction
): SidebarState {
  switch (action.type) {
    case 'SET_ACTIVE_SIDEBAR':
      return { ...state, activeSidebar: action.payload };

    case 'SET_UNIT_CONVERSION_CATEGORY':
      return {
        ...state,
        unitConversion: { ...state.unitConversion, category: action.payload },
      };

    case 'SET_UNIT_CONVERSION_FROM_UNIT':
      return {
        ...state,
        unitConversion: { ...state.unitConversion, fromUnit: action.payload },
      };

    case 'SET_UNIT_CONVERSION_TO_UNIT':
      return {
        ...state,
        unitConversion: { ...state.unitConversion, toUnit: action.payload },
      };

    case 'SET_UNIT_CONVERSION_VALUE':
      return {
        ...state,
        unitConversion: { ...state.unitConversion, value: action.payload },
      };

    case 'SET_EXPORT_SERVICE':
      return {
        ...state,
        services: { ...state.services, exportService: action.payload },
      };

    case 'SET_IMPORT_SERVICE':
      return {
        ...state,
        services: { ...state.services, importService: action.payload },
      };

    case 'RESET_UNIT_CONVERSION':
      return {
        ...state,
        unitConversion: initialState.unitConversion,
      };

    // Quick Plot reducers
    case 'SET_QUICK_PLOT_X_RANGE':
      return {
        ...state,
        quickPlot: { ...state.quickPlot, xRange: action.payload },
      };

    case 'SET_QUICK_PLOT_Y_RANGE':
      return {
        ...state,
        quickPlot: { ...state.quickPlot, yRange: action.payload },
      };

    case 'SET_QUICK_PLOT_ERROR_RANGE':
      return {
        ...state,
        quickPlot: { ...state.quickPlot, errorRange: action.payload },
      };

    case 'SET_QUICK_PLOT_X_LABEL':
      return {
        ...state,
        quickPlot: { ...state.quickPlot, xLabel: action.payload },
      };

    case 'SET_QUICK_PLOT_Y_LABEL':
      return {
        ...state,
        quickPlot: { ...state.quickPlot, yLabel: action.payload },
      };

    case 'SET_QUICK_PLOT_TYPE':
      return {
        ...state,
        quickPlot: { ...state.quickPlot, plotType: action.payload },
      };

    case 'SET_QUICK_PLOT_SHOW_ERROR_BARS':
      return {
        ...state,
        quickPlot: { ...state.quickPlot, showErrorBars: action.payload },
      };

    case 'SET_QUICK_PLOT_EXPORT_THEME':
      return {
        ...state,
        quickPlot: { ...state.quickPlot, exportTheme: action.payload },
      };

    case 'SET_QUICK_PLOT_EXPORT_FORMAT':
      return {
        ...state,
        quickPlot: { ...state.quickPlot, exportFormat: action.payload },
      };

    case 'RESET_QUICK_PLOT':
      return {
        ...state,
        quickPlot: initialQuickPlotState,
      };

    // Uncertainty reducers
    case 'SET_UNCERTAINTY_VARIABLES':
      return {
        ...state,
        uncertainty: { ...state.uncertainty, variables: action.payload },
      };

    case 'ADD_UNCERTAINTY_VARIABLE': {
      const newVariable = action.payload;
      return {
        ...state,
        uncertainty: {
          ...state.uncertainty,
          variables: [...state.uncertainty.variables, newVariable],
        },
      };
    }

    case 'REMOVE_UNCERTAINTY_VARIABLE': {
      const index = action.payload;
      if (state.uncertainty.variables.length <= 1) {
        return state;
      }
      return {
        ...state,
        uncertainty: {
          ...state.uncertainty,
          variables: state.uncertainty.variables.filter((_, i) => i !== index),
        },
      };
    }

    case 'UPDATE_UNCERTAINTY_VARIABLE': {
      const { index, field, value } = action.payload;
      const updated = [...state.uncertainty.variables];
      const currentVar = updated[index];
      if (currentVar) {
        updated[index] = { ...currentVar, [field]: value } as Variable;
      }
      return {
        ...state,
        uncertainty: { ...state.uncertainty, variables: updated },
      };
    }

    case 'SET_UNCERTAINTY_FORMULA':
      return {
        ...state,
        uncertainty: { ...state.uncertainty, formula: action.payload },
      };

    case 'SET_UNCERTAINTY_OUTPUT_VALUE_RANGE':
      return {
        ...state,
        uncertainty: { ...state.uncertainty, outputValueRange: action.payload },
      };

    case 'SET_UNCERTAINTY_OUTPUT_UNCERTAINTY_RANGE':
      return {
        ...state,
        uncertainty: {
          ...state.uncertainty,
          outputUncertaintyRange: action.payload,
        },
      };

    case 'SET_UNCERTAINTY_OUTPUT_CONFIDENCE':
      return {
        ...state,
        uncertainty: { ...state.uncertainty, outputConfidence: action.payload },
      };

    case 'RESET_UNCERTAINTY':
      return {
        ...state,
        uncertainty: initialUncertaintyState,
      };

    // Export sidebar reducers
    case 'SET_EXPORT_FORMAT':
      return {
        ...state,
        export: { ...state.export, format: action.payload },
      };

    case 'SET_EXPORT_RANGE_MODE':
      return {
        ...state,
        export: { ...state.export, rangeMode: action.payload },
      };

    case 'SET_EXPORT_CUSTOM_RANGE':
      return {
        ...state,
        export: { ...state.export, customRange: action.payload },
      };

    case 'SET_EXPORT_CUSTOM_DELIMITER':
      return {
        ...state,
        export: { ...state.export, customDelimiter: action.payload },
      };

    case 'SET_EXPORT_MODE':
      return {
        ...state,
        export: { ...state.export, mode: action.payload },
      };

    case 'SET_EXPORT_LIBRARY_NAME':
      return {
        ...state,
        export: { ...state.export, libraryName: action.payload },
      };

    case 'SET_EXPORT_LIBRARY_DESCRIPTION':
      return {
        ...state,
        export: { ...state.export, libraryDescription: action.payload },
      };

    case 'SET_EXPORT_LIBRARY_TAGS':
      return {
        ...state,
        export: { ...state.export, libraryTags: action.payload },
      };

    case 'SET_EXPORT_LIBRARY_UNIT':
      return {
        ...state,
        export: { ...state.export, libraryUnit: action.payload },
      };

    case 'SET_EXPORT_DATA_RANGE':
      return {
        ...state,
        export: { ...state.export, dataRange: action.payload },
      };

    case 'SET_EXPORT_UNCERTAINTY_RANGE':
      return {
        ...state,
        export: { ...state.export, uncertaintyRange: action.payload },
      };

    case 'RESET_EXPORT':
      return {
        ...state,
        export: initialExportState,
      };

    // Import sidebar reducers
    case 'SET_IMPORT_MODE':
      return {
        ...state,
        import: { ...state.import, mode: action.payload },
      };

    case 'SET_IMPORT_TARGET_RANGE':
      return {
        ...state,
        import: { ...state.import, targetRange: action.payload },
      };

    case 'SET_IMPORT_LIBRARY_DATA_RANGE':
      return {
        ...state,
        import: { ...state.import, libraryDataRange: action.payload },
      };

    case 'SET_IMPORT_LIBRARY_UNCERTAINTY_RANGE':
      return {
        ...state,
        import: { ...state.import, libraryUncertaintyRange: action.payload },
      };

    case 'RESET_IMPORT':
      return {
        ...state,
        import: initialImportState,
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

    // Unit Conversion actions
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

    // Quick Plot actions
    setQuickPlotXRange: useCallback((xRange: string) => {
      dispatch({ type: 'SET_QUICK_PLOT_X_RANGE', payload: xRange });
    }, []),

    setQuickPlotYRange: useCallback((yRange: string) => {
      dispatch({ type: 'SET_QUICK_PLOT_Y_RANGE', payload: yRange });
    }, []),

    setQuickPlotErrorRange: useCallback((errorRange: string) => {
      dispatch({ type: 'SET_QUICK_PLOT_ERROR_RANGE', payload: errorRange });
    }, []),

    setQuickPlotXLabel: useCallback((xLabel: string) => {
      dispatch({ type: 'SET_QUICK_PLOT_X_LABEL', payload: xLabel });
    }, []),

    setQuickPlotYLabel: useCallback((yLabel: string) => {
      dispatch({ type: 'SET_QUICK_PLOT_Y_LABEL', payload: yLabel });
    }, []),

    setQuickPlotType: useCallback((plotType: PlotType) => {
      dispatch({ type: 'SET_QUICK_PLOT_TYPE', payload: plotType });
    }, []),

    setQuickPlotShowErrorBars: useCallback((showErrorBars: boolean) => {
      dispatch({
        type: 'SET_QUICK_PLOT_SHOW_ERROR_BARS',
        payload: showErrorBars,
      });
    }, []),

    setQuickPlotExportTheme: useCallback((exportTheme: ExportTheme) => {
      dispatch({ type: 'SET_QUICK_PLOT_EXPORT_THEME', payload: exportTheme });
    }, []),

    setQuickPlotExportFormat: useCallback(
      (exportFormat: QuickPlotExportFormat) => {
        dispatch({
          type: 'SET_QUICK_PLOT_EXPORT_FORMAT',
          payload: exportFormat,
        });
      },
      []
    ),

    resetQuickPlot: useCallback(() => {
      dispatch({ type: 'RESET_QUICK_PLOT' });
    }, []),

    // Uncertainty actions
    setUncertaintyVariables: useCallback((variables: Variable[]) => {
      dispatch({ type: 'SET_UNCERTAINTY_VARIABLES', payload: variables });
    }, []),

    addUncertaintyVariable: useCallback(() => {
      const nextName = generateNextVariableName(
        state.uncertainty.variables.length
      );
      const newVariable: Variable = {
        name: nextName,
        valueRange: '',
        uncertaintyRange: '',
        confidence: 95,
      };
      dispatch({ type: 'ADD_UNCERTAINTY_VARIABLE', payload: newVariable });
    }, [state.uncertainty.variables.length]),

    removeUncertaintyVariable: useCallback((index: number) => {
      dispatch({ type: 'REMOVE_UNCERTAINTY_VARIABLE', payload: index });
    }, []),

    updateUncertaintyVariable: useCallback(
      (index: number, field: keyof Variable, value: string | number) => {
        dispatch({
          type: 'UPDATE_UNCERTAINTY_VARIABLE',
          payload: { index, field, value },
        });
      },
      []
    ),

    setUncertaintyFormula: useCallback((formula: string) => {
      dispatch({ type: 'SET_UNCERTAINTY_FORMULA', payload: formula });
    }, []),

    setUncertaintyOutputValueRange: useCallback((range: string) => {
      dispatch({ type: 'SET_UNCERTAINTY_OUTPUT_VALUE_RANGE', payload: range });
    }, []),

    setUncertaintyOutputUncertaintyRange: useCallback((range: string) => {
      dispatch({
        type: 'SET_UNCERTAINTY_OUTPUT_UNCERTAINTY_RANGE',
        payload: range,
      });
    }, []),

    setUncertaintyOutputConfidence: useCallback((confidence: number) => {
      dispatch({
        type: 'SET_UNCERTAINTY_OUTPUT_CONFIDENCE',
        payload: confidence,
      });
    }, []),

    resetUncertainty: useCallback(() => {
      dispatch({ type: 'RESET_UNCERTAINTY' });
    }, []),

    // Export sidebar actions
    setExportFormat: useCallback((format: ExportFormat) => {
      dispatch({ type: 'SET_EXPORT_FORMAT', payload: format });
    }, []),

    setExportRangeMode: useCallback((rangeMode: ExportRangeMode) => {
      dispatch({ type: 'SET_EXPORT_RANGE_MODE', payload: rangeMode });
    }, []),

    setExportCustomRange: useCallback((customRange: string) => {
      dispatch({ type: 'SET_EXPORT_CUSTOM_RANGE', payload: customRange });
    }, []),

    setExportCustomDelimiter: useCallback((customDelimiter: string) => {
      dispatch({
        type: 'SET_EXPORT_CUSTOM_DELIMITER',
        payload: customDelimiter,
      });
    }, []),

    setExportMode: useCallback((mode: ExportMode) => {
      dispatch({ type: 'SET_EXPORT_MODE', payload: mode });
    }, []),

    setExportLibraryName: useCallback((libraryName: string) => {
      dispatch({ type: 'SET_EXPORT_LIBRARY_NAME', payload: libraryName });
    }, []),

    setExportLibraryDescription: useCallback((libraryDescription: string) => {
      dispatch({
        type: 'SET_EXPORT_LIBRARY_DESCRIPTION',
        payload: libraryDescription,
      });
    }, []),

    setExportLibraryTags: useCallback((libraryTags: string) => {
      dispatch({ type: 'SET_EXPORT_LIBRARY_TAGS', payload: libraryTags });
    }, []),

    setExportLibraryUnit: useCallback((libraryUnit: string) => {
      dispatch({ type: 'SET_EXPORT_LIBRARY_UNIT', payload: libraryUnit });
    }, []),

    setExportDataRange: useCallback((dataRange: string) => {
      dispatch({ type: 'SET_EXPORT_DATA_RANGE', payload: dataRange });
    }, []),

    setExportUncertaintyRange: useCallback((uncertaintyRange: string) => {
      dispatch({
        type: 'SET_EXPORT_UNCERTAINTY_RANGE',
        payload: uncertaintyRange,
      });
    }, []),

    resetExport: useCallback(() => {
      dispatch({ type: 'RESET_EXPORT' });
    }, []),

    // Import sidebar actions
    setImportMode: useCallback((mode: ImportMode) => {
      dispatch({ type: 'SET_IMPORT_MODE', payload: mode });
    }, []),

    setImportTargetRange: useCallback((targetRange: string) => {
      dispatch({ type: 'SET_IMPORT_TARGET_RANGE', payload: targetRange });
    }, []),

    setImportLibraryDataRange: useCallback((libraryDataRange: string) => {
      dispatch({
        type: 'SET_IMPORT_LIBRARY_DATA_RANGE',
        payload: libraryDataRange,
      });
    }, []),

    setImportLibraryUncertaintyRange: useCallback(
      (libraryUncertaintyRange: string) => {
        dispatch({
          type: 'SET_IMPORT_LIBRARY_UNCERTAINTY_RANGE',
          payload: libraryUncertaintyRange,
        });
      },
      []
    ),

    resetImport: useCallback(() => {
      dispatch({ type: 'RESET_IMPORT' });
    }, []),

    // Convenience method to initialize services
    initializeServices: useCallback(
      (spreadsheetRef: React.RefObject<SpreadsheetRef | null>) => {
        if (spreadsheetRef.current) {
          const expSvc = spreadsheetRef.current.getExportService();
          const impSvc = spreadsheetRef.current.getImportService();
          dispatch({ type: 'SET_EXPORT_SERVICE', payload: expSvc });
          dispatch({ type: 'SET_IMPORT_SERVICE', payload: impSvc });
        }
      },
      []
    ),
  };

  return {
    state,
    actions,
  };
}
