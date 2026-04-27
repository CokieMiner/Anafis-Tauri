// Central state management hook for the Fit tab

import { useCallback, useMemo, useState } from 'react';
import {
  AXES,
  INITIAL_FIT_STATE,
} from '@/tabs/fitting/constants/fittingConstants';
import { FittingService } from '@/tabs/fitting/services/FittingService';
import type {
  AdvancedSettings,
  AxisConfig,
  AxisSettings,
  ColumnMapping,
  DataSourceMode,
  DependentBinding,
  FitStatus,
  ImportedData,
  OdrFitRequest,
  ParameterConfig,
  VariableBinding,
} from '@/tabs/fitting/types/fittingTypes';
import { buildFitRequest } from '@/tabs/fitting/utils/requestBuilder';

/** Try to find a matching uncertainty column name for a given data column */
function findAutoUncColumn(
  dataColName: string,
  allColumnNames: string[]
): string | null {
  const candidates = [
    `σ(${dataColName})`,
    `sigma(${dataColName})`,
    `unc(${dataColName})`,
    `Δ${dataColName}`,
    `d${dataColName}`,
  ];

  const lowerCols = allColumnNames.map((col) => col.toLowerCase());
  for (const candidate of candidates) {
    const idx = lowerCols.indexOf(candidate.toLowerCase());
    if (idx >= 0) {
      return allColumnNames[idx] ?? null;
    }
  }
  return null;
}

function withAxis(
  binding: VariableBinding,
  axis: 'x' | 'y' | 'z' | undefined
): VariableBinding {
  if (axis) {
    return { ...binding, axis };
  }
  const { axis: _axis, ...rest } = binding;
  return rest;
}

export function useFitState() {
  const [state, setState] = useState(INITIAL_FIT_STATE);

  const setDataSourceMode = useCallback((mode: DataSourceMode) => {
    setState((s) => ({ ...s, dataSourceMode: mode }));
  }, []);

  const setImportedData = useCallback((data: ImportedData | null) => {
    setState((s) => {
      if (!data) {
        return { ...s, importedData: null, columnMappings: [] };
      }

      const mappings: ColumnMapping[] = data.columns.map((col, idx) => ({
        columnIndex: idx,
        columnName: col.name,
      }));

      return { ...s, importedData: data, columnMappings: mappings };
    });
  }, []);

  const setColumnMappings = useCallback((mappings: ColumnMapping[]) => {
    setState((s) => ({ ...s, columnMappings: mappings }));
  }, []);

  const setIndependentVarCount = useCallback((count: number) => {
    setState((s) => ({ ...s, independentVarCount: count }));
  }, []);

  const setFormula = useCallback((formula: string) => {
    setState((s) => ({ ...s, customFormula: formula }));
  }, []);

  const setVariableNames = useCallback((names: string[]) => {
    setState((s) => {
      const bindings: VariableBinding[] = names.map((name, idx) => {
        const existing = s.variableBindings.find(
          (binding) => binding.variableName === name
        );
        if (existing) {
          return existing;
        }

        const binding: VariableBinding = {
          variableName: name,
          dataColumn: null,
          uncColumn: null,
          uncertaintyType: 'typeB',
          uncertaintyDegreesOfFreedom: null,
        };

        if (AXES[idx]) {
          binding.axis = AXES[idx];
        }

        return binding;
      });
      return { ...s, variableNames: names, variableBindings: bindings };
    });
  }, []);

  const setParameterNames = useCallback((names: string[]) => {
    setState((s) => {
      const configs: ParameterConfig[] = names.map((name) => {
        const existing = s.parameterConfigs.find((cfg) => cfg.name === name);
        return existing ?? { name, initialValue: 1, fixed: false };
      });

      return {
        ...s,
        parameterNames: names,
        parameterConfigs: configs,
      };
    });
  }, []);

  const setVariableBindings = useCallback((bindings: VariableBinding[]) => {
    setState((s) => ({ ...s, variableBindings: bindings }));
  }, []);

  const updateVariableBinding = useCallback(
    (variableName: string, update: Partial<VariableBinding>) => {
      setState((s) => {
        const colNames = s.importedData?.columns.map((col) => col.name) ?? [];
        const current = s.variableBindings.find(
          (binding) => binding.variableName === variableName
        );
        if (!current) {
          return s;
        }

        const bindings = s.variableBindings.map((binding) => {
          if (binding.variableName !== variableName) {
            return binding;
          }

          const updated = { ...binding, ...update };
          if (
            'dataColumn' in update &&
            update.dataColumn &&
            !('uncColumn' in update)
          ) {
            updated.uncColumn = findAutoUncColumn(update.dataColumn, colNames);
          }
          return updated;
        });

        if (update.axis) {
          const currentAxis = current.axis;
          const targetIdx = bindings.findIndex(
            (binding) => binding.variableName === variableName
          );
          const occupiedIdx = bindings.findIndex(
            (binding, idx) =>
              idx !== targetIdx &&
              binding.variableName !== variableName &&
              binding.axis === update.axis
          );

          if (targetIdx >= 0) {
            const targetBinding = bindings[targetIdx];
            if (targetBinding) {
              bindings[targetIdx] = withAxis(targetBinding, update.axis);
            }
          }
          if (occupiedIdx >= 0) {
            const occupiedBinding = bindings[occupiedIdx];
            if (occupiedBinding) {
              bindings[occupiedIdx] = withAxis(occupiedBinding, currentAxis);
            }
          }
        }

        return { ...s, variableBindings: bindings };
      });
    },
    []
  );

  const setDependentBinding = useCallback((binding: DependentBinding) => {
    setState((s) => ({ ...s, dependentBinding: binding }));
  }, []);

  const updateDependentBinding = useCallback(
    (update: Partial<DependentBinding>) => {
      setState((s) => {
        const colNames = s.importedData?.columns.map((col) => col.name) ?? [];
        const updated = { ...s.dependentBinding, ...update };
        if (
          'dataColumn' in update &&
          update.dataColumn &&
          !('uncColumn' in update)
        ) {
          updated.uncColumn = findAutoUncColumn(update.dataColumn, colNames);
        }
        return { ...s, dependentBinding: updated };
      });
    },
    []
  );

  const setParameterConfigs = useCallback((configs: ParameterConfig[]) => {
    setState((s) => ({ ...s, parameterConfigs: configs }));
  }, []);

  const updateParameterConfig = useCallback(
    (index: number, update: Partial<ParameterConfig>) => {
      setState((s) => {
        const configs = [...s.parameterConfigs];
        const current = configs[index];
        if (!current) {
          return s;
        }
        configs[index] = { ...current, ...update };
        return { ...s, parameterConfigs: configs };
      });
    },
    []
  );

  const setAdvancedSettings = useCallback((settings: AdvancedSettings) => {
    setState((s) => ({ ...s, advancedSettings: settings }));
  }, []);

  const updateAxisConfig = useCallback(
    (axis: keyof AxisSettings, update: Partial<AxisConfig>) => {
      setState((s) => ({
        ...s,
        axisSettings: {
          ...s.axisSettings,
          [axis]: {
            ...s.axisSettings[axis],
            ...update,
          },
        },
      }));
    },
    []
  );

  const setCorrelationMatrices = useCallback(
    (matrices: number[][][] | null) => {
      setState((s) => ({ ...s, correlationMatrices: matrices }));
    },
    []
  );

  const handleAutoEstimate = useCallback(() => {
    setState((s) => {
      const configs = s.parameterConfigs.map((cfg) => ({
        ...cfg,
        initialValue: cfg.fixed ? cfg.initialValue : 1,
      }));
      return { ...s, parameterConfigs: configs };
    });
  }, []);

  const activeFormula = state.customFormula;

  const buildRequest = useCallback((): OdrFitRequest | null => {
    return buildFitRequest(
      state.importedData,
      activeFormula,
      state.variableBindings,
      state.dependentBinding,
      state.parameterConfigs,
      state.advancedSettings,
      state.correlationMatrices
    );
  }, [
    state.importedData,
    activeFormula,
    state.variableBindings,
    state.dependentBinding,
    state.parameterConfigs,
    state.advancedSettings,
    state.correlationMatrices,
  ]);

  const runFit = useCallback(async () => {
    const request = buildRequest();
    if (!request) {
      setState((s) => ({
        ...s,
        fitStatus: 'error' as FitStatus,
        fitError:
          'Cannot build fit request. Check data and model configuration.',
      }));
      return;
    }

    setState((s) => ({
      ...s,
      fitStatus: 'running' as FitStatus,
      fitError: null,
    }));

    try {
      const response = await FittingService.runCustomOdr(request);
      const isSuccess = FittingService.isValidResult(response);

      setState((s) => ({
        ...s,
        fitStatus: isSuccess
          ? ('success' as FitStatus)
          : ('error' as FitStatus),
        fitResult: response,
        fitError: isSuccess ? null : (response.message ?? 'Fit failed'),
      }));
    } catch (err) {
      setState((s) => ({
        ...s,
        fitStatus: 'error' as FitStatus,
        fitError: err instanceof Error ? err.message : String(err),
      }));
    }
  }, [buildRequest]);

  const canRunFit = useMemo(() => {
    if (!state.importedData) {
      return false;
    }

    const allVarsBound =
      state.variableBindings.length > 0 &&
      state.variableBindings.every((binding) => binding.dataColumn !== null);
    const depBound = state.dependentBinding.dataColumn !== null;
    const hasFormula = activeFormula.trim().length > 0;
    const hasParams = state.parameterConfigs.length > 0;

    return allVarsBound && depBound && hasFormula && hasParams;
  }, [
    state.importedData,
    state.variableBindings,
    state.dependentBinding,
    activeFormula,
    state.parameterConfigs,
  ]);

  return {
    state,
    activeFormula,
    canRunFit,

    // Data
    setDataSourceMode,
    setImportedData,
    setColumnMappings,
    setIndependentVarCount,

    // Model
    setFormula,
    setVariableNames,
    setParameterNames,

    // Bindings
    setVariableBindings,
    updateVariableBinding,
    setDependentBinding,
    updateDependentBinding,

    // Parameters
    setParameterConfigs,
    updateParameterConfig,
    setAdvancedSettings,
    updateAxisConfig,
    handleAutoEstimate,

    // Uncertainties
    setCorrelationMatrices,

    // Actions
    runFit,
  };
}
