import { CHART_COLORS, getThemeLayout } from '@/shared/components/plotlyTheme';
import type {
  AxisSettings,
  DependentBinding,
  ImportedData,
  OdrFitResponse,
  VariableBinding,
} from '../types/fittingTypes';

function resolveAxisLabel(
  axisSettings: AxisSettings,
  axis: keyof AxisSettings,
  fallback: string
) {
  const custom = axisSettings[axis].label.trim();
  return custom.length > 0 ? custom : fallback;
}

function formatFitNumber(value: number, digits: number) {
  if (!Number.isFinite(value)) {
    return 'NaN';
  }
  if (value === 0) {
    return '0';
  }
  return value.toPrecision(digits);
}

function buildFitValueLines(fitResult: OdrFitResponse): string[] {
  return fitResult.parameterNames.map((name, idx) => {
    const value = fitResult.parameterValues[idx] ?? 0;
    const uncertainty = fitResult.parameterUncertainties[idx] ?? 0;

    if (Number.isFinite(uncertainty) && Math.abs(uncertainty) > 0) {
      return `${name} = ${formatFitNumber(value, 5)} ± ${formatFitNumber(
        uncertainty,
        2
      )}`;
    }

    return `${name} = ${formatFitNumber(value, 5)}`;
  });
}

function buildFitLegendName(
  fitResult: OdrFitResponse,
  customFormula: string
): string {
  const parameterLines = buildFitValueLines(fitResult);

  const formulaStr = customFormula.trim() || fitResult.formula;

  return [
    `<b>Fit : ${formulaStr}</b>`,
    `χ²red = ${fitResult.chiSquaredReduced.toPrecision(
      4
    )}  |  R² = ${fitResult.rSquared.toPrecision(4)}`,
    ...parameterLines,
  ].join('<br>');
}

function hasUsableFitResult(
  fitResult: OdrFitResponse | null
): fitResult is OdrFitResponse {
  return Boolean(
    fitResult &&
      fitResult.parameterValues.length > 0 &&
      fitResult.fittedValues.length > 0
  );
}

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
          font: { color: '#666', size: 24 },
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
  customFormula: string,
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
    marker: { color: CHART_COLORS.primary, size: 10 },
  };

  // Determine legend position to avoid overlap
  let legendX = 0; // default left
  const minX = Math.min(...xCol.data);
  const maxX = Math.max(...xCol.data);
  const minY = Math.min(...depCol.data);
  const maxY = Math.max(...depCol.data);
  const leftThreshold = minX + 0.5 * (maxX - minX);
  const topThreshold = minY + 0.75 * (maxY - minY);
  const hasDataTopLeft = xCol.data.some(
    (x, i) => x < leftThreshold && (depCol.data[i] ?? 0) > topThreshold
  );
  if (hasDataTopLeft) {
    legendX = 1; // move to right
  }

  if (sigDepCol) {
    scatter.error_y = {
      type: 'data',
      array: sigDepCol.data,
      visible: true,
      thickness: 2.5,
      width: 5.5,
    };
  }

  if (sigXCol) {
    scatter.error_x = {
      type: 'data',
      array: sigXCol.data,
      visible: true,
      thickness: 2.5,
      width: 5.5,
    };
  }

  traces.push(scatter);

  if (hasUsableFitResult(fitResult)) {
    const legendText = buildFitLegendName(fitResult, customFormula);

    const indices = xCol.data
      .map((_, idx) => idx)
      .sort((a, b) => (xCol.data[a] ?? 0) - (xCol.data[b] ?? 0));

    traces.push({
      x: indices.map((idx) => xCol.data[idx] ?? 0),
      y: indices.map((idx) => fitResult.fittedValues[idx] ?? 0),
      mode: 'lines',
      type: 'scatter',
      name: legendText,
      line: { color: CHART_COLORS.fit, width: 2.5, shape: 'spline' },
    });
  }

  return {
    data: traces,
    layout: {
      ...baseLayout,
      showlegend: hasUsableFitResult(fitResult),
      legend: {
        font: { color: theme === 'dark' ? '#aaa' : '#444', size: 18 },
        bgcolor: 'transparent',
        x: legendX === 0 ? 0.02 : 0.98,
        xanchor: legendX === 0 ? 'left' : 'right',
        y: 0.98,
        yanchor: 'top',
      },
      xaxis: {
        ...baseAxis,
        type: axisSettings.x.scale,
        title: {
          text: resolveAxisLabel(axisSettings, 'x', xCol.name),
          font: { color: theme === 'dark' ? '#aaa' : '#444', size: 20 },
        },
      },
      yaxis: {
        ...baseAxis,
        type: axisSettings.y.scale,
        title: {
          text: resolveAxisLabel(axisSettings, 'y', depCol.name),
          font: { color: theme === 'dark' ? '#aaa' : '#444', size: 20 },
        },
      },
      annotations: [],
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
  customFormula: string,
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
      marker: { color: CHART_COLORS.primary, size: 7, opacity: 0.9 },
    },
  ];

  // Determine legend position to avoid overlap
  let legendX = 0; // default left
  const minX = Math.min(...xCol.data);
  const maxX = Math.max(...xCol.data);
  const minZ = Math.min(...depCol.data);
  const maxZ = Math.max(...depCol.data);
  const leftThreshold = minX + 0.5 * (maxX - minX);
  const topThreshold = minZ + 0.75 * (maxZ - minZ);
  const hasDataTopLeft = xCol.data.some(
    (x, i) => x < leftThreshold && (depCol.data[i] ?? 0) > topThreshold
  );
  const hasDataTopRight = xCol.data.some(
    (x, i) => x >= leftThreshold && (depCol.data[i] ?? 0) > topThreshold
  );
  if (hasDataTopLeft && !hasDataTopRight) {
    legendX = 1; // move to right only if data is only on left
  }

  if (hasUsableFitResult(fitResult)) {
    const legendText = buildFitLegendName(fitResult, customFormula);

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
          name: legendText,
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

  return {
    data: traces,
    layout: {
      ...baseLayout,
      showlegend: true,
      legend: {
        font: { color: theme === 'dark' ? '#aaa' : '#444', size: 18 },
        bgcolor: 'transparent',
        x: legendX === 0 ? 0.02 : 0.98,
        xanchor: legendX === 0 ? 'left' : 'right',
        y: 0.98,
        yanchor: 'top',
      },
      scene: {
        xaxis: {
          type: axisSettings.x.scale,
          title: {
            text: resolveAxisLabel(axisSettings, 'x', xCol.name || 'X'),
          },
          ...baseAxis,
          backgroundcolor:
            theme === 'dark' ? 'rgba(14,14,18,0.3)' : 'rgba(255,255,255,0.3)',
        },
        yaxis: {
          type: axisSettings.y.scale,
          title: {
            text: resolveAxisLabel(axisSettings, 'y', yCol.name || 'Y'),
          },
          ...baseAxis,
          backgroundcolor:
            theme === 'dark' ? 'rgba(14,14,18,0.3)' : 'rgba(255,255,255,0.3)',
        },
        zaxis: {
          type: axisSettings.z.scale,
          title: {
            text: resolveAxisLabel(axisSettings, 'z', depCol.name || 'Z'),
          },
          ...baseAxis,
          backgroundcolor:
            theme === 'dark' ? 'rgba(14,14,18,0.3)' : 'rgba(255,255,255,0.3)',
        },
        bgcolor: 'transparent',
      },
      annotations: [],
    },
  };
}

export function buildPredictedChart(
  importedData: ImportedData,
  dependentBinding: DependentBinding,
  axisSettings: AxisSettings,
  fitResult: OdrFitResponse | null,
  customFormula: string,
  theme: 'dark' | 'light' = 'dark'
): { data: Plotly.Data[]; layout: Partial<Plotly.Layout> } {
  const { layout: baseLayout, axis: baseAxis } = getThemeLayout(theme);

  if (!hasUsableFitResult(fitResult)) {
    return {
      data: [],
      layout: {
        ...baseLayout,
        annotations: [
          {
            text: 'Run a fit to see Predicted vs Observed',
            showarrow: false,
            font: { color: '#666', size: 24 },
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

  const legendText = buildFitLegendName(fitResult, customFormula);
  const legendX = 0; // default left

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
        name: legendText,
        marker: { color: CHART_COLORS.primary, size: 10 },
      },
    ],
    layout: {
      ...baseLayout,
      showlegend: true,
      legend: {
        font: { color: theme === 'dark' ? '#aaa' : '#444', size: 10 },
        bgcolor: 'transparent',
        x: legendX,
        y: 1,
      },
      xaxis: {
        ...baseAxis,
        type: axisSettings.x.scale,
        title: {
          text: resolveAxisLabel(axisSettings, 'x', 'Predicted'),
          font: { color: theme === 'dark' ? '#aaa' : '#444', size: 20 },
        },
      },
      yaxis: {
        ...baseAxis,
        type: axisSettings.y.scale,
        title: {
          text: resolveAxisLabel(axisSettings, 'y', 'Observed'),
          font: { color: theme === 'dark' ? '#aaa' : '#444', size: 20 },
        },
      },
      annotations: [],
    },
  };
}

export function buildResidualsChart(
  importedData: ImportedData | null,
  variableBindings: VariableBinding[],
  dependentBinding: DependentBinding,
  axisSettings: AxisSettings,
  fitResult: OdrFitResponse | null,
  theme: 'dark' | 'light' = 'dark'
): { data: Plotly.Data[]; layout: Partial<Plotly.Layout> } {
  const { layout: baseLayout, axis: baseAxis } = getThemeLayout(theme);

  if (!hasUsableFitResult(fitResult)) {
    return {
      data: [],
      layout: {
        ...baseLayout,
        annotations: [
          {
            text: 'Residuals will appear after fitting',
            showarrow: false,
            font: { color: theme === 'dark' ? '#555' : '#999', size: 20 },
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
      (column) => column.name === xBinding.dataColumn
    );
    if (col) {
      xData = col.data;
      xLabel = col.name;
    }
  }

  const residuals = fitResult.residuals;
  if (residuals.length === 0) {
    return {
      data: [],
      layout: {
        ...baseLayout,
        annotations: [
          {
            text: 'No residual data available',
            showarrow: false,
            font: { color: theme === 'dark' ? '#555' : '#999', size: 20 },
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

  const xValues = residuals.map((_, idx) =>
    xData ? (xData[idx] ?? idx) : idx
  );

  const sigYCol = dependentBinding.uncColumn
    ? importedData?.columns.find(
        (column) => column.name === dependentBinding.uncColumn
      )
    : undefined;

  const scatter: Plotly.Data = {
    x: xValues,
    y: residuals,
    mode: 'markers',
    type: 'scatter',
    name: 'Residuals',
    marker: { color: CHART_COLORS.residual, size: 9 },
  };

  if (sigYCol) {
    scatter.error_y = {
      type: 'data',
      array: sigYCol.data,
      visible: true,
      color: 'rgba(239,83,80,0.5)',
      thickness: 2,
      width: 4,
    };
  }

  const minX = Math.min(...xValues);
  const maxX = Math.max(...xValues);

  return {
    data: [
      scatter,
      {
        x: [minX, maxX],
        y: [0, 0],
        mode: 'lines',
        type: 'scatter',
        name: 'Zero',
        line: { color: '#666', width: 1, dash: 'dash' },
        hoverinfo: 'skip',
      },
    ],
    layout: {
      ...baseLayout,
      showlegend: true,
      legend: {
        font: { color: theme === 'dark' ? '#aaa' : '#444', size: 10 },
        bgcolor: 'transparent',
        x: 0.02,
        xanchor: 'left',
        y: 0.98,
        yanchor: 'top',
      },
      margin: { l: 75, r: 24, t: 15, b: 60 },
      xaxis: {
        ...baseAxis,
        type: axisSettings.x.scale,
        title: {
          text: resolveAxisLabel(axisSettings, 'x', xLabel),
          font: { color: theme === 'dark' ? '#aaa' : '#444', size: 18 },
        },
      },
      yaxis: {
        ...baseAxis,
        type: axisSettings.y.scale,
        title: {
          text: resolveAxisLabel(axisSettings, 'y', 'Residual'),
          font: { color: theme === 'dark' ? '#aaa' : '#444', size: 18 },
        },
      },
    },
  };
}
