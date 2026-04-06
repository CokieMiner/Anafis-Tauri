import { CHART_COLORS, getThemeLayout } from '@/shared/components/plotlyTheme';
import type {
  AxisSettings,
  CurveEvaluationResponse,
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

function formatRSquared(value: number): string {
  if (!Number.isFinite(value)) {
    return 'NaN';
  }
  if (value < 1 && value >= 0.9995) {
    return value.toFixed(6);
  }
  return value.toPrecision(6);
}

function compactFormulaLabel(formula: string): string {
  const compact = formula.replace(/\s+/g, ' ').trim();
  if (compact.length <= 40) {
    return compact;
  }
  return `${compact.slice(0, 37)}...`;
}

function buildFitLegendName(
  fitResult: OdrFitResponse,
  customFormula: string
): string {
  const formulaStr = compactFormulaLabel(
    customFormula.trim() || fitResult.formula
  );

  return [
    `<b>Fit (${formulaStr})</b>`,
    `χ²red = ${fitResult.chiSquaredReduced.toPrecision(4)}  |  R² = ${formatRSquared(
      fitResult.rSquared
    )}`,
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

function resolveLegendAnchor(
  xValues: number[],
  yValues: number[]
): 'left' | 'right' {
  const pairs: Array<{ x: number; y: number }> = [];
  for (let idx = 0; idx < xValues.length; idx++) {
    const x = xValues[idx] ?? Number.NaN;
    const y = yValues[idx] ?? Number.NaN;
    if (Number.isFinite(x) && Number.isFinite(y)) {
      pairs.push({ x, y });
    }
  }

  if (pairs.length < 4) {
    return 'left';
  }

  const xs = pairs.map((point) => point.x);
  const ys = pairs.map((point) => point.y);
  const minX = Math.min(...xs);
  const maxX = Math.max(...xs);
  const minY = Math.min(...ys);
  const maxY = Math.max(...ys);

  const xMid = minX + 0.5 * (maxX - minX);
  const topBandStart = minY + 0.7 * (maxY - minY);

  let topLeftCount = 0;
  let topRightCount = 0;
  for (const point of pairs) {
    if (point.y >= topBandStart) {
      if (point.x < xMid) {
        topLeftCount += 1;
      } else {
        topRightCount += 1;
      }
    }
  }

  // Put the legend on the least crowded top side.
  return topLeftCount <= topRightCount ? 'left' : 'right';
}

function legendPosition(
  xValues: number[],
  yValues: number[]
): {
  x: number;
  xanchor: 'left' | 'right';
  y: number;
  yanchor: 'top';
} {
  const anchor = resolveLegendAnchor(xValues, yValues);
  return {
    x: anchor === 'left' ? 0.02 : 0.98,
    xanchor: anchor,
    y: 0.98,
    yanchor: 'top',
  };
}

function uncertaintyBarColor(theme: 'dark' | 'light'): string {
  return theme === 'dark' ? '#3f8fd6' : '#2b6ea9';
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
  curveData: CurveEvaluationResponse | null,
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
    marker: { color: CHART_COLORS.primary, size: 11 },
  };
  const legend = legendPosition(xCol.data, depCol.data);

  if (sigDepCol) {
    scatter.error_y = {
      type: 'data',
      array: sigDepCol.data,
      visible: true,
      color: uncertaintyBarColor(theme),
      thickness: 2.5,
      width: 5.5,
    };
  }

  if (sigXCol) {
    scatter.error_x = {
      type: 'data',
      array: sigXCol.data,
      visible: true,
      color: uncertaintyBarColor(theme),
      thickness: 2.5,
      width: 5.5,
    };
  }

  traces.push(scatter);

  if (hasUsableFitResult(fitResult)) {
    const legendText = buildFitLegendName(fitResult, customFormula);

    const curveFromFormula =
      curveData &&
      curveData.x.length > 1 &&
      curveData.x.length === curveData.y.length;

    const indices = curveFromFormula
      ? []
      : xCol.data
          .map((_, idx) => idx)
          .sort((a, b) => (xCol.data[a] ?? 0) - (xCol.data[b] ?? 0));

    traces.push({
      x: curveFromFormula
        ? curveData.x
        : indices.map((idx) => xCol.data[idx] ?? 0),
      y: curveFromFormula
        ? curveData.y
        : indices.map((idx) => fitResult.fittedValues[idx] ?? 0),
      mode: 'lines',
      type: 'scatter',
      name: legendText,
      // Keep rendering geometric, without smoothing artifacts.
      line: { color: CHART_COLORS.fit, width: 2.5, shape: 'linear' },
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
        ...legend,
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
      marker: {
        color: CHART_COLORS.primary,
        size: 8,
        opacity: 0.95,
      },
    },
  ];
  const legend = legendPosition(xCol.data, depCol.data);

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
        ...legend,
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
  const legend = legendPosition(fitResult.fittedValues, observed.data);

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
        marker: { color: CHART_COLORS.primary, size: 11 },
      },
    ],
    layout: {
      ...baseLayout,
      showlegend: true,
      legend: {
        font: { color: theme === 'dark' ? '#aaa' : '#444', size: 10 },
        bgcolor: 'transparent',
        ...legend,
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
  const legend = legendPosition(xValues, residuals);

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
        ...legend,
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
