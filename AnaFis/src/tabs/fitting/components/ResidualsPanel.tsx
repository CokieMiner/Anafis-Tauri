// Residuals Panel — Plotly scatter of residuals with zero line and error bars

import { Box, Typography } from '@mui/material';
import { useMemo } from 'react';
import Plot from '@/shared/components/PlotlyChart';
import {
  ANAFIS_CHART_CONFIG,
  ANAFIS_DARK_AXIS,
  ANAFIS_DARK_LAYOUT,
  CHART_COLORS,
} from '@/shared/components/plotlyTheme';

import type {
  AxisSettings,
  DependentBinding,
  ImportedData,
  OdrFitResponse,
  VariableBinding,
} from '../types/fittingTypes';

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
  const { data, layout } = useMemo((): {
    data: Plotly.Data[];
    layout: Partial<Plotly.Layout>;
  } => {
    if (!fitResult?.success) {
      return {
        data: [],
        layout: {
          ...ANAFIS_DARK_LAYOUT,
          annotations: [
            {
              text: 'Residuals will appear after fitting',
              showarrow: false,
              font: { color: '#555', size: 12 },
              xref: 'paper',
              yref: 'paper',
              x: 0.5,
              y: 0.5,
            },
          ],
          xaxis: { visible: false },
          yaxis: { visible: false },
        },
      };
    }

    let xData: number[] | null = null;
    let xLabel = 'Index';

    const xBinding =
      variableBindings.find((binding) => binding.axis?.toLowerCase() === 'x') ??
      variableBindings[0];

    if (importedData && xBinding?.dataColumn) {
      const col = importedData.columns.find(
        (c) => c.name === xBinding.dataColumn
      );
      if (col) {
        xData = col.data;
        xLabel = col.name;
      }
    }

    const residuals = fitResult.residuals;
    const xValues = residuals.map((_, idx) =>
      xData ? (xData[idx] ?? idx) : idx
    );

    const sigYCol = dependentBinding.uncColumn
      ? importedData?.columns.find((c) => c.name === dependentBinding.uncColumn)
      : undefined;

    const scatter: Plotly.Data = {
      x: xValues,
      y: residuals,
      mode: 'markers',
      type: 'scatter',
      name: 'Residuals',
      marker: { color: CHART_COLORS.residual, size: 5 },
    };

    if (sigYCol) {
      scatter.error_y = {
        type: 'data',
        array: sigYCol.data,
        visible: true,
        color: 'rgba(239,83,80,0.5)',
        thickness: 1,
        width: 2,
      };
    }

    return {
      data: [
        scatter,
        {
          x: [Math.min(...xValues), Math.max(...xValues)],
          y: [0, 0],
          mode: 'lines',
          type: 'scatter',
          name: 'Zero',
          line: { color: '#666', width: 1, dash: 'dash' },
          hoverinfo: 'skip',
        },
      ],
      layout: {
        ...ANAFIS_DARK_LAYOUT,
        margin: { l: 50, r: 10, t: 15, b: 40 },
        xaxis: {
          ...ANAFIS_DARK_AXIS,
          type: axisSettings.x.scale,
          title: {
            text:
              axisSettings.x.label.trim().length > 0
                ? axisSettings.x.label
                : xLabel,
            font: { color: '#aaa', size: 10 },
          },
        },
        yaxis: {
          ...ANAFIS_DARK_AXIS,
          type: axisSettings.y.scale,
          title: {
            text:
              axisSettings.y.label.trim().length > 0
                ? axisSettings.y.label
                : 'Residual',
            font: { color: '#aaa', size: 10 },
          },
        },
      },
    };
  }, [
    fitResult,
    importedData,
    variableBindings,
    dependentBinding,
    axisSettings,
  ]);

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
