import { Box } from '@mui/material';
import { createTheme, ThemeProvider, useTheme } from '@mui/material/styles';
import { useMemo } from 'react';
import { useDataLibrary } from '@/shared/dataLibrary/managers/useDataLibraryManager';
import { anafisTheme } from '@/shared/theme/unifiedTheme';
import AxisSettingsSection from './components/AxisSettingsSection';
import BindingsSection from './components/BindingsSection';
import DataSourceSection from './components/DataSourceSection';
import FitSettingsSection from './components/FitSettingsSection';
import FitVisualization from './components/FitVisualization';
import ModelSection from './components/ModelSection';
import ResidualsPanel from './components/ResidualsPanel';
import ResultsPanel from './components/ResultsPanel';
import { useFitState } from './hooks/useFitState';

const PANEL_GAP = 2;

const FittingTab = () => {
  const baseTheme = useTheme();

  // Use unified theme colors for fitting tab
  const fittingTheme = useMemo(
    () =>
      createTheme(baseTheme, {
        palette: {
          primary: {
            main: anafisTheme.colors.tabs.fitting.main,
            light: anafisTheme.colors.tabs.fitting.light,
            dark: anafisTheme.colors.tabs.fitting.dark,
            contrastText: anafisTheme.colors.tabs.fitting.contrast,
          },
          warning: {
            main: anafisTheme.colors.tabs.fitting.main,
            light: anafisTheme.colors.tabs.fitting.light,
            dark: anafisTheme.colors.tabs.fitting.dark,
            contrastText: anafisTheme.colors.tabs.fitting.contrast,
          },
        },
      }),
    [baseTheme]
  );

  const fit = useFitState();
  const { sequences: librarySequences } = useDataLibrary();
  const visualizationMode = useMemo(() => {
    const varCount = fit.state.variableBindings.length;
    if (varCount === 0) {
      return 'empty' as const;
    }
    if (varCount === 1) {
      return '2d' as const;
    }
    if (varCount === 2) {
      return '3d' as const;
    }
    return 'predicted' as const;
  }, [fit.state.variableBindings.length]);

  return (
    <ThemeProvider theme={fittingTheme}>
      <Box
        sx={{
          height: '100%',
          p: 2,
          boxSizing: 'border-box',
          background: anafisTheme.gradients.backgroundRadialFitting,
          overflow: 'auto',
        }}
      >
        <Box
          sx={{
            height: '100%',
            minWidth: 1024,
            display: 'grid',
            gap: PANEL_GAP,
            gridTemplateAreas: `
              'control visual'
              'results residuals'
            `,
            gridTemplateColumns: 'minmax(320px, 25%) minmax(0, 75%)',
            gridTemplateRows: 'minmax(0, 3fr) minmax(0, 1fr)',
            minHeight: 0,
          }}
        >
          {/* ── Control Panel (scrollable) ── */}
          <Box
            sx={{
              gridArea: 'control',
              minHeight: 0,
              overflow: 'auto',
              pr: 0.5,
              '&::-webkit-scrollbar': { width: 5 },
              '&::-webkit-scrollbar-thumb': {
                background: `${anafisTheme.colors.tabs.fitting.main}4D`,
                borderRadius: 4,
              },
            }}
          >
            <DataSourceSection
              mode={fit.state.dataSourceMode}
              importedData={fit.state.importedData}
              librarySequences={librarySequences}
              onModeChange={fit.setDataSourceMode}
              onDataImported={fit.setImportedData}
            />

            <ModelSection
              formula={fit.state.customFormula}
              variableNames={fit.state.variableNames}
              parameterNames={fit.state.parameterNames}
              onFormulaChange={fit.setFormula}
              onVariableNamesChange={fit.setVariableNames}
              onParameterNamesChange={fit.setParameterNames}
            />

            <FitSettingsSection
              parameterConfigs={fit.state.parameterConfigs}
              advancedSettings={fit.state.advancedSettings}
              onUpdateParameterConfig={fit.updateParameterConfig}
              onUpdateAdvancedSettings={fit.setAdvancedSettings}
            />

            <BindingsSection
              importedData={fit.state.importedData}
              variableNames={fit.state.variableNames}
              variableBindings={fit.state.variableBindings}
              dependentBinding={fit.state.dependentBinding}
              onUpdateVariableBinding={fit.updateVariableBinding}
              onUpdateDependentBinding={fit.updateDependentBinding}
            />

            <AxisSettingsSection
              axisSettings={fit.state.axisSettings}
              onUpdateAxisConfig={fit.updateAxisConfig}
              mode={visualizationMode}
            />
          </Box>

          {/* ── Visualization ── */}
          <Box sx={{ gridArea: 'visual', minHeight: 0, overflow: 'hidden' }}>
            <FitVisualization
              importedData={fit.state.importedData}
              variableBindings={fit.state.variableBindings}
              dependentBinding={fit.state.dependentBinding}
              fitResult={fit.state.fitResult}
              axisSettings={fit.state.axisSettings}
            />
          </Box>

          {/* ── Results ── */}
          <Box sx={{ gridArea: 'results', minHeight: 0, overflow: 'hidden' }}>
            <ResultsPanel
              fitResult={fit.state.fitResult}
              canRunFit={fit.canRunFit}
              fitStatus={fit.state.fitStatus}
              fitError={fit.state.fitError}
              onRunFit={() => {
                void fit.runFit();
              }}
            />
          </Box>

          {/* ── Residuals ── */}
          <Box sx={{ gridArea: 'residuals', minHeight: 0, overflow: 'hidden' }}>
            <ResidualsPanel
              fitResult={fit.state.fitResult}
              importedData={fit.state.importedData}
              variableBindings={fit.state.variableBindings}
              dependentBinding={fit.state.dependentBinding}
              axisSettings={fit.state.axisSettings}
            />
          </Box>
        </Box>
      </Box>
    </ThemeProvider>
  );
};

export default FittingTab;
