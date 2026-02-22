import { Box, Typography } from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import Plot from '@/shared/components/PlotlyChart';
import { ANAFIS_CHART_CONFIG } from '@/shared/components/plotlyTheme';

import type {
  AxisSettings,
  DependentBinding,
  GridEvaluationResponse,
  ImportedData,
  OdrFitResponse,
  VariableBinding,
} from '../types/fittingTypes';
import {
  build2DChart,
  build3DChart,
  buildEmptyChart,
  buildPredictedChart,
} from '../utils/chartBuilders';

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
      return buildEmptyChart();
    }

    if (mode === '2d') {
      return build2DChart(
        importedData,
        variableBindings,
        dependentBinding,
        axisSettings,
        fitResult
      );
    }

    if (mode === '3d') {
      return build3DChart(
        importedData,
        variableBindings,
        dependentBinding,
        axisSettings,
        fitResult,
        gridData
      );
    }

    return buildPredictedChart(
      importedData,
      dependentBinding,
      axisSettings,
      fitResult
    );
  }, [
    mode,
    importedData,
    variableBindings,
    dependentBinding,
    fitResult,
    gridData,
    axisSettings,
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
