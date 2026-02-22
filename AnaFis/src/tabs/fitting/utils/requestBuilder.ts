import type {
  AdvancedSettings,
  DependentBinding,
  ImportedData,
  OdrFitRequest,
  ParameterConfig,
  VariableBinding,
  VariableInput,
} from '../types/fittingTypes';

export function buildFitRequest(
  importedData: ImportedData | null,
  activeFormula: string,
  variableBindings: VariableBinding[],
  dependentBinding: DependentBinding,
  parameterConfigs: ParameterConfig[],
  advancedSettings: AdvancedSettings,
  correlationMatrices: number[][][] | null
): OdrFitRequest | null {
  if (!importedData) {
    return null;
  }

  const colByName = (name: string | null) => {
    if (!name) {
      return undefined;
    }
    return importedData.columns.find((col) => col.name === name);
  };

  const yCol = colByName(dependentBinding.dataColumn);
  if (!yCol) {
    return null;
  }
  const sigmaYCol = colByName(dependentBinding.uncColumn);

  const independentVariables: VariableInput[] = [];
  const layerIndependentNames: string[] = [];

  for (const binding of variableBindings) {
    const col = colByName(binding.dataColumn);
    if (!col) {
      return null;
    }

    const input: VariableInput = {
      name: binding.variableName,
      values: col.data,
    };

    const uncCol = colByName(binding.uncColumn);
    if (uncCol) {
      input.uncertainties = uncCol.data;
    }

    independentVariables.push(input);
    layerIndependentNames.push(binding.variableName);
  }

  if (independentVariables.length === 0) {
    return null;
  }

  const dependentName =
    yCol.name.trim().length > 0 ? yCol.name.toLowerCase() : 'y';

  const dependentInput: VariableInput = {
    name: dependentName,
    values: yCol.data,
  };

  if (sigmaYCol) {
    dependentInput.uncertainties = sigmaYCol.data;
  }

  const request: OdrFitRequest = {
    layers: [
      {
        formula: activeFormula,
        dependentVariable: dependentName,
        independentVariables: layerIndependentNames,
      },
    ],
    dependentVariables: [dependentInput],
    independentVariables,
    parameterNames: parameterConfigs.filter((p) => !p.fixed).map((p) => p.name),
    initialGuess: parameterConfigs
      .filter((p) => !p.fixed)
      .map((p) => p.initialValue),
    maxIterations: advancedSettings.maxIterations,
  };

  // Temporarily commented out for single-layer frontend, ready to be wired to UI later
  // request.usePoissonWeighting = undefined;

  if (correlationMatrices) {
    request.pointCorrelations = correlationMatrices;
  }

  return request;
}
