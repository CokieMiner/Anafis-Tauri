// Fitting Tab TypeScript Types
// Mirrors Rust backend structures for `fit_custom_odr`

// ─── Backend mirror types ────────────────────────────────────────────

export interface IndependentVariableInput {
  name: string;
  values: number[];
  uncertainties?: number[];
}

export interface OdrFitRequest {
  modelFormula: string;
  dependentVariable: string;
  independentVariables: IndependentVariableInput[];
  observedValues: number[];
  observedUncertainties?: number[];
  parameterNames: string[];
  initialGuess?: number[];
  maxIterations?: number;
  pointCorrelations?: number[][][]; // [point][dim][dim]
}

export interface OdrFitResponse {
  success: boolean;
  message?: string;
  iterations: number;
  formula: string;
  dependentVariable: string;
  independentVariables: string[];
  parameterNames: string[];
  parameterValues: number[];
  parameterUncertainties: number[];
  parameterCovariance: number[][]; // Full covariance matrix
  residuals: number[];
  fittedValues: number[];
  chiSquared: number;
  chiSquaredReduced: number;
  rmse: number;
  rSquared: number;
}

export interface GridEvaluationRequest {
  modelFormula: string;
  independentNames: string[];
  parameterNames: string[];
  parameterValues: number[];
  xRange: [number, number];
  yRange: [number, number];
  resolution: number;
}

export interface GridEvaluationResponse {
  x: number[];
  y: number[];
  z: number[];
}

// ─── Frontend UI types ──────────────────────────────────────────────

/** Binding an independent variable from the formula to a data column */
export interface VariableBinding {
  variableName: string; // from formula parser: "x", "y", "z", etc.
  dataColumn: string | null; // column name from imported data
  uncColumn: string | null; // uncertainty column name (auto or manual)
  axis?: 'x' | 'y' | 'z'; // chart axis assignment (auto for 1 var, user-chosen for >1)
}

/** Binding for the dependent variable (the observed output being fitted) */
export interface DependentBinding {
  dataColumn: string | null; // column of observed values
  uncColumn: string | null; // column of uncertainties on observed values
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

export const DEFAULT_ADVANCED_SETTINGS: AdvancedSettings = {
  maxIterations: 200,
  tolerance: 1e-9,
  initialDamping: 1e-3,
};

export const DEFAULT_AXIS_SETTINGS: AxisSettings = {
  x: { label: '', scale: 'linear' },
  y: { label: '', scale: 'linear' },
  z: { label: '', scale: 'linear' },
};

export const DEFAULT_CSV_SETTINGS: CsvImportSettings = {
  separator: 'auto',
  decimalFormat: '.',
  skipRows: 0,
  hasHeader: true,
};
