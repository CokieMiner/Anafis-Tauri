import {
  CHART_COLORS,
  getThemeLayout,
} from '@/shared/components/plotlyTheme';
import type {
  AxisSettings,
  DependentBinding,
  ImportedData,
  OdrFitResponse,
  VariableBinding,
} from '../types/fittingTypes';

export function buildEmptyChart(theme: 'dark' | 'light' = 'dark') {
  const { layout: baseLayout } = getThemeLayout(theme);
  return {
    data: [],
    layout: {
      ...baseLayout,
      annotations: [
        {
          text: 'Import data and run a fit',
          showarrow: false,
          font: { color: '#666', size: 14 },
          xref: 'paper' as const,
          yref: 'paper' as const,
          x: 0.5,
          y: 0.5,
        },
      ],
      xaxis: { visible: false },
      yaxis: { visible: false },
    },
  };
}

export function build2DChart(
  importedData: ImportedData,
  variableBindings: VariableBinding[],
  dependentBinding: DependentBinding,
  axisSettings: AxisSettings,
  fitResult: OdrFitResponse | null,
  theme: 'dark' | 'light' = 'dark'
): { data: Plotly.Data[]; layout: Partial<Plotly.Layout> } {
  const { layout: baseLayout, axis: baseAxis } = getThemeLayout(theme);

  const colByName = (name: string | null) =>
    name ? importedData.columns.find((c) => c.name === name) : undefined;

  const depCol = colByName(dependentBinding.dataColumn);
  const xBinding = variableBindings[0];
  const xCol = xBinding ? colByName(xBinding.dataColumn) : undefined;

  if (!depCol || !xCol) {
    return { data: [], layout: baseLayout };
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
      thickness: 1.5,
      width: 3,
    };
  }

  if (sigXCol) {
    scatter.error_x = {
      type: 'data',
      array: sigXCol.data,
      visible: true,
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


  const axisLabel = (axis: keyof AxisSettings, fallback: string) => {
    const custom = axisSettings[axis].label.trim();
    return custom.length > 0 ? custom : fallback;
  };

  return {
    data: traces,
    layout: {
      ...baseLayout,
      xaxis: {
        ...baseAxis,
        type: axisSettings.x.scale,
        title: {
          text: axisLabel('x', xCol.name),
          font: { color: theme === 'dark' ? '#aaa' : '#444', size: 12 },
        },
      },
      yaxis: {
        ...baseAxis,
        type: axisSettings.y.scale,
        title: {
          text: axisLabel('y', depCol.name),
          font: { color: theme === 'dark' ? '#aaa' : '#444', size: 12 },
        },
      },
    },
  };
}

export function build3DChart(
  importedData: ImportedData,
  variableBindings: VariableBinding[],
  dependentBinding: DependentBinding,
  axisSettings: AxisSettings,
  fitResult: OdrFitResponse | null,
  gridData: { x: number[]; y: number[]; z: number[] } | null,
  theme: 'dark' | 'light' = 'dark'
): { data: Plotly.Data[]; layout: Partial<Plotly.Layout> } {
  const { layout: baseLayout, axis: baseAxis } = getThemeLayout(theme);

  const colByName = (name: string | null) =>
    name ? importedData.columns.find((c) => c.name === name) : undefined;

  const depCol = colByName(dependentBinding.dataColumn);
  const xBinding =
    variableBindings.find((binding) => binding.axis?.toLowerCase() === 'x') ??
    variableBindings[0];
  const yBinding =
    variableBindings.find((binding) => binding.axis?.toLowerCase() === 'y') ??
    variableBindings[1];

  const xCol = xBinding ? colByName(xBinding.dataColumn) : undefined;
  const yCol = yBinding ? colByName(yBinding.dataColumn) : undefined;

  if (!depCol || !xCol || !yCol) {
    return { data: [], layout: baseLayout };
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
      xref: 'paper' as const,
      yref: 'paper' as const,
      x: 1,
      y: -0.1,
      xanchor: 'right' as const,
      yanchor: 'top' as const,
    });
  }

  const axisLabel = (axis: keyof AxisSettings, fallback: string) => {
    const custom = axisSettings[axis].label.trim();
    return custom.length > 0 ? custom : fallback;
  };

  return {
    data: traces,
    layout: {
      ...baseLayout,
      showlegend: true,
      legend: {
        font: { color: theme === 'dark' ? '#aaa' : '#444', size: 10 },
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
          ...baseAxis,
          backgroundcolor: theme === 'dark' ? 'rgba(14,14,18,0.3)' : 'rgba(255,255,255,0.3)',
        },
        yaxis: {
          type: axisSettings.y.scale,
          title: {
            text: axisLabel('y', yCol.name || 'Y'),
          },
          ...baseAxis,
          backgroundcolor: theme === 'dark' ? 'rgba(14,14,18,0.3)' : 'rgba(255,255,255,0.3)',
        },
        zaxis: {
          type: axisSettings.z.scale,
          title: {
            text: axisLabel('z', depCol.name || 'Z'),
          },
          ...baseAxis,
          backgroundcolor: theme === 'dark' ? 'rgba(14,14,18,0.3)' : 'rgba(255,255,255,0.3)',
        },
        bgcolor: 'transparent',
      },
      margin: {
        ...baseLayout.margin,
        b: 40, // Increase bottom margin for annotation
      },
      annotations,
    },
  };
}

export function buildPredictedChart(
  importedData: ImportedData,
  dependentBinding: DependentBinding,
  axisSettings: AxisSettings,
  fitResult: OdrFitResponse | null,
  theme: 'dark' | 'light' = 'dark'
): { data: Plotly.Data[]; layout: Partial<Plotly.Layout> } {
  const { layout: baseLayout, axis: baseAxis } = getThemeLayout(theme);

  if (!fitResult?.success) {
    return {
      data: [],
      layout: {
        ...baseLayout,
        annotations: [
          {
            text: 'Run a fit to see Predicted vs Observed',
            showarrow: false,
            font: { color: '#666', size: 14 },
            xref: 'paper' as const,
            yref: 'paper' as const,
            x: 0.5,
            y: 0.5,
          },
        ],
        xaxis: { visible: false },
        yaxis: { visible: false },
      },
    };
  }

  const colByName = (name: string | null) =>
    name ? importedData.columns.find((c) => c.name === name) : undefined;
  const observed = colByName(dependentBinding.dataColumn);

  if (!observed) {
    return { data: [], layout: baseLayout };
  }

  const allVals = [...fitResult.fittedValues, ...observed.data];
  const lo = Math.min(...allVals);
  const hi = Math.max(...allVals);
  const pad = (hi - lo) * 0.05;

  const axisLabel = (axis: keyof AxisSettings, fallback: string) => {
    const custom = axisSettings[axis].label.trim();
    return custom.length > 0 ? custom : fallback;
  };

  return {
    data: [
      {
        x: [lo - pad, hi + pad],
        y: [lo - pad, hi + pad],
        mode: 'lines',
        type: 'scatter',
        name: '1:1',
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
      ...baseLayout,
      xaxis: {
        ...baseAxis,
        type: axisSettings.x.scale,
        title: {
          text: axisLabel('x', 'Predicted'),
          font: { color: theme === 'dark' ? '#aaa' : '#444', size: 12 },
        },
      },
      yaxis: {
        ...baseAxis,
        type: axisSettings.y.scale,
        title: {
          text: axisLabel('y', 'Observed'),
          font: { color: theme === 'dark' ? '#aaa' : '#444', size: 12 },
        },
      },
      annotations: [
        {
          text: `χ²red = ${fitResult.chiSquaredReduced.toPrecision(4)}  |  R² = ${fitResult.rSquared.toPrecision(4)}`,
          showarrow: false,
          font: { color: theme === 'dark' ? '#aaa' : '#444', size: 11, family: 'monospace' },
          xref: 'paper' as const,
          yref: 'paper' as const,
          x: 1,
          y: -0.15,
          xanchor: 'right' as const,
          yanchor: 'top' as const,
        },
      ],
    },
  };
}
