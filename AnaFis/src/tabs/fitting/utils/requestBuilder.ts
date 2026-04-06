import type {
  AdvancedSettings,
  DependentBinding,
  ImportedData,
  OdrFitRequest,
  ParameterConfig,
  VariableBinding,
  VariableInput,
} from '../types/fittingTypes';

export function parseFormula(formula: string): {
  dependentVariable?: string;
  formulaRhs: string;
} {
  const lines = formula.split('\n');
  const firstLine = lines[0]?.trim() ?? '';
  const equalsIndex = firstLine.indexOf('=');
  if (equalsIndex !== -1) {
    const dependentVariable = firstLine.substring(0, equalsIndex).trim();
    const formulaRhs = firstLine.substring(equalsIndex + 1).trim();
    return { dependentVariable, formulaRhs };
  }
  return { formulaRhs: firstLine };
}

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

  const columnsByName = new Map(
    importedData.columns.map((column) => [column.name, column] as const)
  );

  const colByName = (name: string | null) => {
    if (!name) {
      return undefined;
    }
    return columnsByName.get(name);
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

    if (binding.uncertaintyType !== null) {
      input.uncertaintyType = binding.uncertaintyType;
    }
    if (
      binding.uncertaintyDegreesOfFreedom !== null &&
      Number.isFinite(binding.uncertaintyDegreesOfFreedom)
    ) {
      input.uncertaintyDegreesOfFreedom = binding.uncertaintyDegreesOfFreedom;
    }

    independentVariables.push(input);
    layerIndependentNames.push(binding.variableName);
  }

  if (independentVariables.length === 0) {
    return null;
  }

  const { dependentVariable: formulaDependent, formulaRhs } =
    parseFormula(activeFormula);
  // Use the dependent variable name from the formula if present, otherwise fall back to column name
  const dependentName =
    formulaDependent ??
    (yCol.name.trim().length > 0 ? yCol.name.toLowerCase() : 'y');

  const dependentInput: VariableInput = {
    name: dependentName,
    values: yCol.data,
  };

  if (sigmaYCol) {
    dependentInput.uncertainties = sigmaYCol.data;
  }

  if (dependentBinding.uncertaintyType !== null) {
    dependentInput.uncertaintyType = dependentBinding.uncertaintyType;
  }
  if (
    dependentBinding.uncertaintyDegreesOfFreedom !== null &&
    Number.isFinite(dependentBinding.uncertaintyDegreesOfFreedom)
  ) {
    dependentInput.uncertaintyDegreesOfFreedom =
      dependentBinding.uncertaintyDegreesOfFreedom;
  }

  const request: OdrFitRequest = {
    layers: [
      {
        formula: formulaRhs,
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
    tolerance: advancedSettings.tolerance,
    initialDamping: advancedSettings.initialDamping,
  };
  if (advancedSettings.usePoissonWeighting !== undefined) {
    request.usePoissonWeighting = advancedSettings.usePoissonWeighting;
  }

  if (correlationMatrices) {
    request.pointCorrelations = correlationMatrices;
  }

  return request;
}
