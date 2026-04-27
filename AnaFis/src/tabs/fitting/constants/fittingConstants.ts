import type {
  AdvancedSettings,
  AxisSettings,
  DependentBinding,
  FitState,
} from '@/tabs/fitting/types/fittingTypes';

export const AXES: Array<'x' | 'y' | 'z'> = ['x', 'y', 'z'] as const;

const DEFAULT_ADVANCED_SETTINGS: AdvancedSettings = {
  maxIterations: 200,
  tolerance: 1e-9,
  initialDamping: 1e-3,
  usePoissonWeighting: false,
};

const DEFAULT_AXIS_SETTINGS: AxisSettings = {
  x: { label: '', scale: 'linear' },
  y: { label: '', scale: 'linear' },
  z: { label: '', scale: 'linear' },
};

const DEFAULT_DEPENDENT: DependentBinding = {
  dataColumn: null,
  uncColumn: null,
  uncertaintyType: 'typeB',
  uncertaintyDegreesOfFreedom: null,
};

export const INITIAL_FIT_STATE: FitState = {
  dataSourceMode: 'library',
  importedData: null,
  columnMappings: [],
  independentVarCount: 1,

  customFormula: '',
  variableNames: [],
  parameterNames: [],

  variableBindings: [],
  dependentBinding: { ...DEFAULT_DEPENDENT },

  parameterConfigs: [],
  advancedSettings: { ...DEFAULT_ADVANCED_SETTINGS },
  axisSettings: {
    x: { ...DEFAULT_AXIS_SETTINGS.x },
    y: { ...DEFAULT_AXIS_SETTINGS.y },
    z: { ...DEFAULT_AXIS_SETTINGS.z },
  },

  correlationMatrices: null,

  fitStatus: 'idle',
  fitError: null,
  fitResult: null,
};
