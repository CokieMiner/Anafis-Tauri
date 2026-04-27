// Fitting Tab TypeScript Types
// Mirrors Rust backend structures for `fit_custom_odr`

// ─── Backend mirror types ────────────────────────────────────────────

interface ModelLayer {
  formula: string;
  dependentVariable: string;
  independentVariables: string[];
}

type UncertaintyType = 'typeA' | 'typeB';

export interface VariableInput {
  name: string;
  values: number[];
  uncertainties?: number[];
  uncertaintyType?: UncertaintyType;
  uncertaintyDegreesOfFreedom?: number;
}

export interface OdrFitRequest {
  layers: ModelLayer[];
  independentVariables: VariableInput[];
  dependentVariables: VariableInput[];
  usePoissonWeighting?: boolean;
  parameterNames: string[];
  initialGuess?: number[];
  maxIterations?: number;
  tolerance?: number;
  initialDamping?: number;
  confidenceLevel?: number;
  pointCorrelations?: number[][][]; // [point][dim][dim]
}

export interface OdrFitResponse {
  success: boolean;
  terminationReason: string;
  message?: string;
  iterations: number;
  formula: string;
  dependentVariable: string;
  independentVariables: string[];
  parameterNames: string[];
  parameterValues: number[];
  parameterUncertainties: number[];
  parameterUncertaintiesRaw: number[];
  parameterExpandedUncertainties: number[];
  coverageFactor: number;
  parameterCovariance: number[][]; // Full covariance matrix
  parameterCovarianceRaw: number[][]; // Full covariance matrix (raw)
  parameterCorrelations: number[][];
  parameterCorrelationsRaw: number[][];
  residuals: number[];
  fittedValues: number[];
  chiSquared: number;
  chiSquaredObservation: number;
  chiSquaredObservationReduced: number;
  chiSquaredReduced: number;
  rmse: number;
  residualStandardError: number;
  rSquared: number;
  rSquaredPerLayer: number[];
  effectiveRank: number;
  conditionNumber: number;
  innerStationarityNormMax: number;
  innerStationarityNormMean: number;
  welchSatterthwaiteDof?: number;
  coverageDegreesOfFreedom?: number;
  assumptions: string[];
}

export interface GridEvaluationResponse {
  x: number[];
  y: number[];
  z: number[];
}

export interface CurveEvaluationResponse {
  x: number[];
  y: number[];
}

// ─── Frontend UI types ──────────────────────────────────────────────

/** Binding an independent variable from the formula to a data column */
export interface VariableBinding {
  variableName: string; // from formula parser: "x", "y", "z", etc.
  dataColumn: string | null; // column name from imported data
  uncColumn: string | null; // uncertainty column name (auto or manual)
  uncertaintyType: UncertaintyType | null;
  uncertaintyDegreesOfFreedom: number | null;
  axis?: 'x' | 'y' | 'z'; // chart axis assignment (auto for 1 var, user-chosen for >1)
}

/** Binding for the dependent variable (the observed output being fitted) */
export interface DependentBinding {
  dataColumn: string | null; // column of observed values
  uncColumn: string | null; // column of uncertainties on observed values
  uncertaintyType: UncertaintyType | null;
  uncertaintyDegreesOfFreedom: number | null;
}

export type DataSourceMode = 'library' | 'csv';

export interface ColumnMapping {
  columnIndex: number;
  columnName: string;
  /** For multi-dimensional: which independent variable index (0, 1, 2...) */
  independentIndex?: number;
}

export interface ImportedColumn {
  name: string;
  data: number[];
}

export interface ImportedData {
  columns: ImportedColumn[];
  sourceName: string;
  rowCount: number;
}

export interface ParameterConfig {
  name: string;
  initialValue: number;
  fixed: boolean;
}

export interface AdvancedSettings {
  maxIterations: number;
  tolerance: number;
  initialDamping: number;
  usePoissonWeighting?: boolean;
}

export interface CsvImportSettings {
  separator: ',' | ';' | '\t' | 'auto';
  decimalFormat: '.' | ',';
  skipRows: number;
  hasHeader: boolean;
}

export type AxisScale = 'linear' | 'log';

export interface AxisConfig {
  label: string;
  scale: AxisScale;
}

export interface AxisSettings {
  x: AxisConfig;
  y: AxisConfig;
  z: AxisConfig;
}
export type FitStatus = 'idle' | 'running' | 'success' | 'error';

export interface FitState {
  // Data
  dataSourceMode: DataSourceMode;
  importedData: ImportedData | null;
  columnMappings: ColumnMapping[];
  independentVarCount: number;

  // Model
  customFormula: string;
  variableNames: string[];
  parameterNames: string[];

  // Bindings (variables → data columns)
  variableBindings: VariableBinding[];
  dependentBinding: DependentBinding;

  // Parameters
  parameterConfigs: ParameterConfig[];
  advancedSettings: AdvancedSettings;
  axisSettings: AxisSettings;

  // Uncertainties
  correlationMatrices: number[][][] | null; // [point][dim][dim]

  // Execution
  fitStatus: FitStatus;
  fitError: string | null;

  // Results
  fitResult: OdrFitResponse | null;
}

export const DEFAULT_CSV_SETTINGS: CsvImportSettings = {
  separator: 'auto',
  decimalFormat: '.',
  skipRows: 0,
  hasHeader: true,
};
