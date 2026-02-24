// Residuals Panel — Plotly scatter of residuals with zero line and error bars

import { Box, Typography } from '@mui/material';
import { useMemo } from 'react';
import Plot from '@/shared/components/PlotlyChart';
import { ANAFIS_CHART_CONFIG } from '@/shared/components/plotlyTheme';

import type {
  AxisSettings,
  DependentBinding,
  ImportedData,
  OdrFitResponse,
  VariableBinding,
} from '../types/fittingTypes';
import { buildResidualsChart } from '../utils/chartBuilders';

interface ResidualsPanelProps {
  fitResult: OdrFitResponse | null;
  importedData: ImportedData | null;
  variableBindings: VariableBinding[];
  dependentBinding: DependentBinding;
  axisSettings: AxisSettings;
}

export default function ResidualsPanel({
  fitResult,
  importedData,
  variableBindings,
  dependentBinding,
  axisSettings,
}: ResidualsPanelProps) {
  const { data, layout } = useMemo(
    (): {
      data: Plotly.Data[];
      layout: Partial<Plotly.Layout>;
    } =>
      buildResidualsChart(
        importedData,
        variableBindings,
        dependentBinding,
        axisSettings,
        fitResult
      ),
    [fitResult, importedData, variableBindings, dependentBinding, axisSettings]
  );

  return (
    <Box
      sx={{
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        p: 1.5,
        borderRadius: 2,
        border: '1px solid rgba(148, 163, 184, 0.18)',
        background:
          'linear-gradient(140deg, rgba(19, 19, 24, 0.95) 0%, rgba(14, 14, 18, 0.9) 100%)',
      }}
    >
      <Typography
        variant="subtitle2"
        sx={{ fontWeight: 600, mb: 0.5, color: 'text.secondary' }}
      >
        Residuals
      </Typography>
      <Box sx={{ flex: 1, minHeight: 120 }}>
        <Plot
          data={data}
          layout={layout}
          config={ANAFIS_CHART_CONFIG}
          useResizeHandler
          style={{ width: '100%', height: '100%' }}
        />
      </Box>
    </Box>
  );
}
