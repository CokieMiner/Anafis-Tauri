import { Box, Typography } from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
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
  GridEvaluationResponse,
  ImportedData,
  OdrFitResponse,
  VariableBinding,
} from '../types/fittingTypes';

interface FitVisualizationProps {
  importedData: ImportedData | null;
  variableBindings: VariableBinding[];
  dependentBinding: DependentBinding;
  fitResult: OdrFitResponse | null;
  axisSettings: AxisSettings;
}

export default function FitVisualization({
  importedData,
  variableBindings,
  dependentBinding,
  fitResult,
  axisSettings,
}: FitVisualizationProps) {
  const [gridData, setGridData] = useState<GridEvaluationResponse | null>(null);
  const gridRequestRef = useRef(0);

  const colByName = useCallback(
    (name: string | null) => {
      if (!name || !importedData) {
        return undefined;
      }
      return importedData.columns.find((col) => col.name === name);
    },
    [importedData]
  );

  const varCount = variableBindings.length;
  const mode: '2d' | '3d' | 'predicted' | 'empty' = useMemo(() => {
    if (varCount === 0) {
      return 'empty';
    }
    if (varCount === 1) {
      return '2d';
    }
    if (varCount === 2) {
      return '3d';
    }
    return 'predicted';
  }, [varCount]);

  const axisLabel = useCallback(
    (axis: keyof AxisSettings, fallback: string) => {
      const custom = axisSettings[axis].label.trim();
      return custom.length > 0 ? custom : fallback;
    },
    [axisSettings]
  );

  useEffect(() => {
    const requestId = gridRequestRef.current + 1;
    gridRequestRef.current = requestId;

    if (mode !== '3d' || !fitResult?.success) {
      queueMicrotask(() => {
        if (gridRequestRef.current === requestId) {
          setGridData(null);
        }
      });
      return;
    }

    const xBinding =
      variableBindings.find((binding) => binding.axis === 'x') ??
      variableBindings[0];
    const yBinding =
      variableBindings.find((binding) => binding.axis === 'y') ??
      variableBindings[1];
    if (!xBinding || !yBinding) {
      queueMicrotask(() => {
        if (gridRequestRef.current === requestId) {
          setGridData(null);
        }
      });
      return;
    }
    const xCol = colByName(xBinding.dataColumn);
    const yCol = colByName(yBinding.dataColumn);

    if (!xCol || !yCol || xCol.data.length === 0 || yCol.data.length === 0) {
      queueMicrotask(() => {
        if (gridRequestRef.current === requestId) {
          setGridData(null);
        }
      });
      return;
    }

    const xVals = xCol.data;
    const yVals = yCol.data;
    const xMin = Math.min(...xVals);
    const xMax = Math.max(...xVals);
    const yMin = Math.min(...yVals);
    const yMax = Math.max(...yVals);
    const padX = (xMax - xMin) * 0.1 || 1.0;
    const padY = (yMax - yMin) * 0.1 || 1.0;

    void invoke<GridEvaluationResponse>('evaluate_model_grid', {
      request: {
        modelFormula: fitResult.formula,
        independentNames: [xBinding.variableName, yBinding.variableName],
        parameterNames: fitResult.parameterNames,
        parameterValues: fitResult.parameterValues,
        xRange: [xMin - padX, xMax + padX],
        yRange: [yMin - padY, yMax + padY],
        resolution: 30,
      },
    })
      .then((response) => {
        if (gridRequestRef.current === requestId) {
          setGridData(response);
        }
      })
      .catch((error: unknown) => {
        console.error('Failed to evaluate 3D grid:', error);
        if (gridRequestRef.current === requestId) {
          setGridData(null);
        }
      });
  }, [mode, fitResult, variableBindings, colByName]);

  const { data, layout } = useMemo((): {
    data: Plotly.Data[];
    layout: Partial<Plotly.Layout>;
  } => {
    if (mode === 'empty' || !importedData) {
      return {
        data: [],
        layout: {
          ...ANAFIS_DARK_LAYOUT,
          annotations: [
            {
              text: 'Import data and run a fit',
              showarrow: false,
              font: { color: '#666', size: 14 },
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

    if (mode === '2d') {
      const depCol = colByName(dependentBinding.dataColumn);
      const xBinding = variableBindings[0];
      const xCol = xBinding ? colByName(xBinding.dataColumn) : undefined;

      if (!depCol || !xCol) {
        return { data: [], layout: ANAFIS_DARK_LAYOUT };
      }

      const sigDepCol = colByName(dependentBinding.uncColumn);
      const sigXCol = xBinding ? colByName(xBinding.uncColumn) : undefined;

      const traces: Plotly.Data[] = [];
      const scatter: Plotly.Data = {
        x: xCol.data,
        y: depCol.data,
        mode: 'markers',
        type: 'scatter',
        name: 'Data',
        marker: { color: CHART_COLORS.primary, size: 6 },
      };

      if (sigDepCol) {
        scatter.error_y = {
          type: 'data',
          array: sigDepCol.data,
          visible: true,
          color: 'rgba(100,181,246,0.6)',
          thickness: 1.5,
          width: 3,
        };
      }

      if (sigXCol) {
        scatter.error_x = {
          type: 'data',
          array: sigXCol.data,
          visible: true,
          color: 'rgba(100,181,246,0.4)',
          thickness: 1.5,
          width: 3,
        };
      }

      traces.push(scatter);

      if (fitResult?.success) {
        const indices = xCol.data
          .map((_, idx) => idx)
          .sort((a, b) => (xCol.data[a] ?? 0) - (xCol.data[b] ?? 0));

        traces.push({
          x: indices.map((idx) => xCol.data[idx] ?? 0),
          y: indices.map((idx) => fitResult.fittedValues[idx] ?? 0),
          mode: 'lines',
          type: 'scatter',
          name: 'Fit',
          line: { color: CHART_COLORS.fit, width: 2.5, shape: 'spline' },
        });
      }

      const annotations: Plotly.Layout['annotations'] = [];
      if (fitResult?.success) {
        annotations.push({
          text: `χ²red = ${fitResult.chiSquaredReduced.toPrecision(4)}  |  R² = ${fitResult.rSquared.toPrecision(4)}`,
          showarrow: false,
          font: { color: '#aaa', size: 11, family: 'monospace' },
          xref: 'paper',
          yref: 'paper',
          x: 1,
          y: 1.05,
          xanchor: 'right',
        });
      }

      return {
        data: traces,
        layout: {
          ...ANAFIS_DARK_LAYOUT,
          xaxis: {
            ...ANAFIS_DARK_AXIS,
            type: axisSettings.x.scale,
            title: {
              text: axisLabel('x', xCol.name),
              font: { color: '#aaa', size: 12 },
            },
          },
          yaxis: {
            ...ANAFIS_DARK_AXIS,
            type: axisSettings.y.scale,
            title: {
              text: axisLabel('y', depCol.name),
              font: { color: '#aaa', size: 12 },
            },
          },
          annotations,
        },
      };
    }

    if (mode === '3d') {
      const depCol = colByName(dependentBinding.dataColumn);
      const xBinding =
        variableBindings.find(
          (binding) => binding.axis?.toLowerCase() === 'x'
        ) ?? variableBindings[0];
      const yBinding =
        variableBindings.find(
          (binding) => binding.axis?.toLowerCase() === 'y'
        ) ?? variableBindings[1];

      const xCol = xBinding ? colByName(xBinding.dataColumn) : undefined;
      const yCol = yBinding ? colByName(yBinding.dataColumn) : undefined;

      if (!depCol || !xCol || !yCol) {
        return { data: [], layout: ANAFIS_DARK_LAYOUT };
      }

      const traces: Plotly.Data[] = [
        {
          x: xCol.data,
          y: yCol.data,
          z: depCol.data,
          mode: 'markers',
          type: 'scatter3d',
          name: 'Data',
          marker: { color: CHART_COLORS.primary, size: 4, opacity: 0.9 },
        },
      ];

      if (fitResult?.success) {
        if (gridData && gridData.z.length > 0) {
          const res = Math.round(Math.sqrt(gridData.z.length));
          if (res * res === gridData.z.length) {
            const zMatrix: number[][] = [];
            for (let row = 0; row < res; row++) {
              zMatrix.push(gridData.z.slice(row * res, (row + 1) * res));
            }

            const xUnique = gridData.x.slice(0, res);
            const yUnique = Array.from(
              { length: res },
              (_, row) => gridData.y[row * res] ?? 0
            );

            traces.push({
              x: xUnique,
              y: yUnique,
              z: zMatrix,
              type: 'surface',
              name: 'Fitted surface',
              opacity: 0.6,
              colorscale: [
                [0, CHART_COLORS.fit],
                [1, CHART_COLORS.fit],
              ],
              showscale: false,
              showlegend: true,
              lighting: {
                ambient: 0.6,
                diffuse: 0.8,
                fresnel: 0.2,
                specular: 0.05,
                roughness: 0.1,
              },
            } as Plotly.Data);
          }
        }

        const needleX: Array<number | null> = [];
        const needleY: Array<number | null> = [];
        const needleZ: Array<number | null> = [];

        for (let idx = 0; idx < depCol.data.length; idx++) {
          needleX.push(xCol.data[idx] ?? null, xCol.data[idx] ?? null, null);
          needleY.push(yCol.data[idx] ?? null, yCol.data[idx] ?? null, null);
          needleZ.push(
            depCol.data[idx] ?? null,
            fitResult.fittedValues[idx] ?? null,
            null
          );
        }

        traces.push({
          x: needleX,
          y: needleY,
          z: needleZ,
          mode: 'lines',
          type: 'scatter3d',
          name: 'Residuals',
          line: { color: CHART_COLORS.error, width: 1.5 },
          opacity: 0.5,
          showlegend: true,
        } as Plotly.Data);
      }

      const annotations: Plotly.Layout['annotations'] = [];
      if (fitResult?.success) {
        annotations.push({
          text: `χ²red = ${fitResult.chiSquaredReduced.toPrecision(4)}  |  R² = ${fitResult.rSquared.toPrecision(4)}`,
          showarrow: false,
          font: { color: '#aaa', size: 11, family: 'monospace' },
          xref: 'paper',
          yref: 'paper',
          x: 1,
          y: 1.02,
          xanchor: 'right',
        });
      }

      return {
        data: traces,
        layout: {
          ...ANAFIS_DARK_LAYOUT,
          showlegend: true,
          legend: {
            font: { color: '#aaa', size: 10 },
            bgcolor: 'transparent',
            x: 0,
            y: 1,
          },
          scene: {
            xaxis: {
              type: axisSettings.x.scale,
              title: {
                text: axisLabel('x', xCol.name || 'X'),
              },
              ...ANAFIS_DARK_AXIS,
              backgroundcolor: 'rgba(14,14,18,0.3)',
            },
            yaxis: {
              type: axisSettings.y.scale,
              title: {
                text: axisLabel('y', yCol.name || 'Y'),
              },
              ...ANAFIS_DARK_AXIS,
              backgroundcolor: 'rgba(14,14,18,0.3)',
            },
            zaxis: {
              type: axisSettings.z.scale,
              title: {
                text: axisLabel('z', depCol.name || 'Z'),
              },
              ...ANAFIS_DARK_AXIS,
              backgroundcolor: 'rgba(14,14,18,0.3)',
            },
            bgcolor: 'transparent',
          },
          annotations,
        },
      };
    }

    if (fitResult?.success) {
      const observed = colByName(dependentBinding.dataColumn);
      if (!observed) {
        return { data: [], layout: ANAFIS_DARK_LAYOUT };
      }

      const allVals = [...fitResult.fittedValues, ...observed.data];
      const lo = Math.min(...allVals);
      const hi = Math.max(...allVals);
      const pad = (hi - lo) * 0.05;

      return {
        data: [
          {
            x: [lo - pad, hi + pad],
            y: [lo - pad, hi + pad],
            mode: 'lines',
            type: 'scatter',
            name: '1:1',
            line: { color: 'rgba(255,179,0,0.4)', width: 1.5, dash: 'dash' },
          },
          {
            x: fitResult.fittedValues,
            y: observed.data,
            mode: 'markers',
            type: 'scatter',
            name: 'Predicted vs Observed',
            marker: { color: CHART_COLORS.primary, size: 6 },
          },
        ],
        layout: {
          ...ANAFIS_DARK_LAYOUT,
          xaxis: {
            ...ANAFIS_DARK_AXIS,
            type: axisSettings.x.scale,
            title: {
              text: axisLabel('x', 'Predicted'),
              font: { color: '#aaa', size: 12 },
            },
          },
          yaxis: {
            ...ANAFIS_DARK_AXIS,
            type: axisSettings.y.scale,
            title: {
              text: axisLabel('y', 'Observed'),
              font: { color: '#aaa', size: 12 },
            },
          },
          annotations: [
            {
              text: `χ²red = ${fitResult.chiSquaredReduced.toPrecision(4)}  |  R² = ${fitResult.rSquared.toPrecision(4)}`,
              showarrow: false,
              font: { color: '#aaa', size: 11, family: 'monospace' },
              xref: 'paper',
              yref: 'paper',
              x: 1,
              y: 1.05,
              xanchor: 'right',
            },
          ],
        },
      };
    }

    return {
      data: [],
      layout: {
        ...ANAFIS_DARK_LAYOUT,
        annotations: [
          {
            text: 'Run a fit to see Predicted vs Observed',
            showarrow: false,
            font: { color: '#666', size: 14 },
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
  }, [
    mode,
    importedData,
    variableBindings,
    dependentBinding,
    fitResult,
    gridData,
    colByName,
    axisSettings,
    axisLabel,
  ]);

  const modeLabel =
    mode === '3d'
      ? '3D Visualization'
      : mode === 'predicted'
        ? 'Predicted vs Observed'
        : 'Visualization';

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
        {modeLabel}
      </Typography>
      <Box sx={{ flex: 1, minHeight: 200 }}>
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
