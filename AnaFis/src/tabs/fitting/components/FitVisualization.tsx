import ImageIcon from '@mui/icons-material/Image';
import {
  Box,
  Button,
  Checkbox,
  CircularProgress,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  FormControl,
  FormControlLabel,
  IconButton,
  Radio,
  RadioGroup,
  Tooltip,
  Typography,
} from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import Plot, { Plotly } from '@/shared/components/PlotlyChart';
import { ANAFIS_CHART_CONFIG } from '@/shared/components/plotlyTheme';
import { anafisTheme } from '@/shared/theme/unifiedTheme';
import { saveWithMemory } from '@/shared/utils/dialogMemory';
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
  buildResidualsChart,
} from '../utils/chartBuilders';

interface FitVisualizationProps {
  importedData: ImportedData | null;
  variableBindings: VariableBinding[];
  dependentBinding: DependentBinding;
  fitResult: OdrFitResponse | null;
  axisSettings: AxisSettings;
  customFormula: string;
}

function assignCartesianAxes(
  traces: Plotly.Data[],
  xaxis: 'x' | 'x2',
  yaxis: 'y' | 'y2'
): Plotly.Data[] {
  return traces.map(
    (trace) =>
      ({
        ...(trace as Record<string, unknown>),
        xaxis,
        yaxis,
      }) as Plotly.Data
  );
}

function getAnnotations(
  annotations: Plotly.Layout['annotations'] | undefined
): Partial<Plotly.Annotations>[] {
  return Array.isArray(annotations)
    ? (annotations as Partial<Plotly.Annotations>[])
    : [];
}

export default function FitVisualization({
  importedData,
  variableBindings,
  dependentBinding,
  fitResult,
  axisSettings,
  customFormula,
}: FitVisualizationProps) {
  const [gridData, setGridData] = useState<GridEvaluationResponse | null>(null);
  const gridRequestRef = useRef(0);
  const hasUsableFitResult =
    Boolean(fitResult) &&
    (fitResult?.parameterValues.length ?? 0) > 0 &&
    (fitResult?.fittedValues.length ?? 0) > 0;

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
    const resolvedFitResult = fitResult;

    if (mode !== '3d' || !hasUsableFitResult || !resolvedFitResult) {
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
        modelFormula: resolvedFitResult.formula,
        independentNames: [xBinding.variableName, yBinding.variableName],
        parameterNames: resolvedFitResult.parameterNames,
        parameterValues: resolvedFitResult.parameterValues,
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
  }, [mode, fitResult, variableBindings, colByName, hasUsableFitResult]);

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
        fitResult,
        customFormula
      );
    }

    if (mode === '3d') {
      return build3DChart(
        importedData,
        variableBindings,
        dependentBinding,
        axisSettings,
        fitResult,
        gridData,
        customFormula
      );
    }

    return buildPredictedChart(
      importedData,
      dependentBinding,
      axisSettings,
      fitResult,
      customFormula
    );
  }, [
    mode,
    importedData,
    variableBindings,
    dependentBinding,
    fitResult,
    gridData,
    axisSettings,
    customFormula,
  ]);

  const modeLabel =
    mode === '3d'
      ? '3D Visualization'
      : mode === 'predicted'
        ? 'Predicted vs Observed'
        : 'Visualization';

  const [exportDialogOpen, setExportDialogOpen] = useState(false);
  const [exportFormat, setExportFormat] = useState<'png' | 'svg'>('png');
  const [exportTheme, setExportTheme] = useState<'dark' | 'light'>('dark');
  const [includeResidualsInExport, setIncludeResidualsInExport] =
    useState(false);
  const [isExporting, setIsExporting] = useState(false);
  const canIncludeResiduals = Boolean(
    hasUsableFitResult && importedData && mode !== 'empty'
  );

  useEffect(() => {
    if (!canIncludeResiduals) {
      setIncludeResidualsInExport(false);
    }
  }, [canIncludeResiduals]);

  const handleExport = useCallback(async () => {
    if (mode === 'empty' || !importedData) {
      return;
    }
    setIsExporting(true);
    try {
      const extension = exportFormat;
      const filterName = exportFormat === 'svg' ? 'SVG Image' : 'PNG Image';

      const filePath = await saveWithMemory({
        defaultPath: `fit_plot_${Date.now()}.${extension}`,
        filters: [{ name: filterName, extensions: [extension] }],
      });

      if (!filePath) {
        setIsExporting(false);
        setExportDialogOpen(false);
        return;
      }

      // Generate plot data for the requested theme
      let exportData: Plotly.Data[] = [];
      let exportLayout: Partial<Plotly.Layout> = {};

      if (mode === '2d') {
        const result = build2DChart(
          importedData,
          variableBindings,
          dependentBinding,
          axisSettings,
          fitResult,
          customFormula,
          exportTheme
        );
        exportData = result.data;
        exportLayout = result.layout;
      } else if (mode === '3d') {
        const result = build3DChart(
          importedData,
          variableBindings,
          dependentBinding,
          axisSettings,
          fitResult,
          gridData,
          customFormula,
          exportTheme
        );
        exportData = result.data;
        exportLayout = result.layout;
      } else if (mode === 'predicted') {
        const result = buildPredictedChart(
          importedData,
          dependentBinding,
          axisSettings,
          fitResult,
          customFormula,
          exportTheme
        );
        exportData = result.data;
        exportLayout = result.layout;
      } else {
        exportData = data;
        exportLayout = layout;
      }

      const shouldIncludeResiduals =
        includeResidualsInExport && canIncludeResiduals;
      const exportWidth = 1200;
      let exportHeight = 800;
      let finalData = exportData;
      let finalLayout = exportLayout;

      if (shouldIncludeResiduals) {
        const residualDomain: [number, number] = [0, 0.2];
        const mainDomain: [number, number] = [0.22, 1];

        const residualResult = buildResidualsChart(
          importedData,
          variableBindings,
          dependentBinding,
          axisSettings,
          fitResult,
          exportTheme
        );
        const residualData = assignCartesianAxes(residualResult.data, 'x', 'y');
        const mainAnnotations = getAnnotations(exportLayout.annotations);
        const residualAnnotations = getAnnotations(
          residualResult.layout.annotations
        );

        exportHeight = 1000;

        if (mode === '3d') {
          finalData = [...exportData, ...residualData];
          finalLayout = {
            ...exportLayout,
            scene: {
              ...(exportLayout.scene ?? {}),
              domain: { x: [0, 1], y: mainDomain },
            },
            xaxis: {
              ...((residualResult.layout.xaxis ??
                {}) as Partial<Plotly.LayoutAxis>),
              domain: [0, 1],
              anchor: 'y',
            },
            yaxis: {
              ...((residualResult.layout.yaxis ??
                {}) as Partial<Plotly.LayoutAxis>),
              domain: residualDomain,
              anchor: 'x',
            },
            annotations: [...mainAnnotations, ...residualAnnotations],
            margin: {
              ...(exportLayout.margin ?? {}),
              t: 40,
              b: 60,
            },
          };
        } else {
          const mainX = (exportLayout.xaxis ??
            {}) as Partial<Plotly.LayoutAxis>;
          const mainY = (exportLayout.yaxis ??
            {}) as Partial<Plotly.LayoutAxis>;

          finalData = [
            ...assignCartesianAxes(exportData, 'x2', 'y2'),
            ...residualData,
          ];
          finalLayout = {
            ...exportLayout,
            xaxis: {
              ...((residualResult.layout.xaxis ??
                {}) as Partial<Plotly.LayoutAxis>),
              domain: [0, 1],
              anchor: 'y',
            },
            yaxis: {
              ...((residualResult.layout.yaxis ??
                {}) as Partial<Plotly.LayoutAxis>),
              domain: residualDomain,
              anchor: 'x',
            },
            xaxis2: {
              ...mainX,
              domain: [0, 1],
              anchor: 'y2',
              title: {
                ...((mainX.title as Partial<Plotly.Title>) ?? {}),
                text: '', // Remove title from the top plot
              },
            },
            yaxis2: {
              ...mainY,
              domain: mainDomain,
              anchor: 'x2',
            },
            annotations: [...mainAnnotations, ...residualAnnotations],
            margin: {
              ...(exportLayout.margin ?? {}),
              t: 40,
              b: 60,
            },
          };
        }
      }

      const tempDiv = document.createElement('div');
      tempDiv.style.width = `${exportWidth}px`;
      tempDiv.style.height = `${exportHeight}px`;
      tempDiv.style.position = 'absolute';
      tempDiv.style.left = '-9999px';
      document.body.appendChild(tempDiv);

      try {
        const layoutForExport = {
          ...finalLayout,
          width: exportWidth,
          height: exportHeight,
        };

        // Enforce solid background for dark exports so they aren't transparent
        if (exportTheme === 'dark') {
          layoutForExport.paper_bgcolor = '#0e0e12';
        }

        await Plotly.newPlot(tempDiv, finalData, layoutForExport);

        if (exportFormat === 'svg') {
          const svgDataUrl = await Plotly.toImage(tempDiv, {
            format: 'svg',
            width: exportWidth,
            height: exportHeight,
          });

          let svgString: string;
          if (svgDataUrl.includes(';base64,')) {
            const base64 = svgDataUrl.split(';base64,')[1] ?? '';
            svgString = atob(base64);
          } else {
            const encoded = svgDataUrl.split(',').slice(1).join(',');
            svgString = decodeURIComponent(encoded);
          }

          await invoke('save_svg_file', {
            svgContent: svgString,
            path: filePath,
          });
        } else {
          const dataURL = await Plotly.toImage(tempDiv, {
            format: 'png',
            width: exportWidth,
            height: exportHeight,
            scale: 2,
          });

          await invoke('save_image_from_data_url', {
            dataUrl: dataURL,
            path: filePath,
          });
        }

        Plotly.purge(tempDiv);
      } finally {
        document.body.removeChild(tempDiv);
      }
      setExportDialogOpen(false);
    } catch (error) {
      console.error('Export failed:', error);
    } finally {
      setIsExporting(false);
    }
  }, [
    mode,
    exportFormat,
    exportTheme,
    includeResidualsInExport,
    canIncludeResiduals,
    importedData,
    variableBindings,
    dependentBinding,
    axisSettings,
    fitResult,
    gridData,
    data,
    layout,
    customFormula,
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
      <Box sx={{ display: 'flex', alignItems: 'center', mb: 0.5, gap: 1 }}>
        <Typography
          variant="subtitle2"
          sx={{ fontWeight: 600, color: 'text.secondary' }}
        >
          {modeLabel}
        </Typography>
        <Tooltip title="Export Plot">
          <span>
            <IconButton
              size="small"
              onClick={() => setExportDialogOpen(true)}
              disabled={isExporting || mode === 'empty'}
              sx={{ color: 'text.secondary', p: 0.25 }}
            >
              {isExporting ? (
                <CircularProgress size={16} />
              ) : (
                <ImageIcon fontSize="small" />
              )}
            </IconButton>
          </span>
        </Tooltip>
      </Box>
      <Box sx={{ flex: 1, minHeight: 200 }}>
        <Plot
          data={data}
          layout={layout}
          config={ANAFIS_CHART_CONFIG}
          useResizeHandler
          style={{ width: '100%', height: '100%' }}
        />
      </Box>
      <Dialog
        open={exportDialogOpen}
        onClose={() => setExportDialogOpen(false)}
        maxWidth="xs"
        fullWidth
        slotProps={{
          paper: {
            sx: {
              bgcolor: anafisTheme.colors.background.elevated,
              backgroundImage: 'none',
              color: anafisTheme.colors.text.primary,
            },
          },
        }}
      >
        <DialogTitle
          sx={{
            color: anafisTheme.colors.tabs.fitting.main,
            borderBottom: `1px solid ${anafisTheme.colors.border.light}`,
          }}
        >
          Export Plot
        </DialogTitle>
        <DialogContent sx={{ pt: 3 }}>
          <Typography
            variant="body2"
            sx={{ color: anafisTheme.colors.text.secondary, mb: 2 }}
          >
            Choose export format, theme, and content:
          </Typography>

          <Typography
            variant="caption"
            sx={{
              color: anafisTheme.colors.text.tertiary,
              fontSize: 10,
              fontWeight: 600,
              mb: 0.5,
              display: 'block',
            }}
          >
            FORMAT
          </Typography>
          <FormControl sx={{ mb: 2 }}>
            <RadioGroup
              value={exportFormat}
              onChange={(e) => setExportFormat(e.target.value as 'png' | 'svg')}
            >
              <FormControlLabel
                value="png"
                control={
                  <Radio
                    size="small"
                    sx={{
                      color: 'rgba(255, 179, 0, 0.5)',
                      '&.Mui-checked': {
                        color: anafisTheme.colors.tabs.fitting.main,
                      },
                    }}
                  />
                }
                label={
                  <Typography
                    sx={{
                      fontSize: 13,
                      color: anafisTheme.colors.text.primary,
                    }}
                  >
                    PNG (Raster)
                  </Typography>
                }
              />
              <FormControlLabel
                value="svg"
                control={
                  <Radio
                    size="small"
                    sx={{
                      color: 'rgba(255, 179, 0, 0.5)',
                      '&.Mui-checked': {
                        color: anafisTheme.colors.tabs.fitting.main,
                      },
                    }}
                  />
                }
                label={
                  <Typography
                    sx={{
                      fontSize: 13,
                      color: anafisTheme.colors.text.primary,
                    }}
                  >
                    SVG (Vector)
                  </Typography>
                }
              />
            </RadioGroup>
          </FormControl>

          <Typography
            variant="caption"
            sx={{
              color: anafisTheme.colors.text.tertiary,
              fontSize: 10,
              fontWeight: 600,
              mb: 0.5,
              display: 'block',
            }}
          >
            THEME
          </Typography>
          <FormControl sx={{ mb: 2 }}>
            <RadioGroup
              value={exportTheme}
              onChange={(e) =>
                setExportTheme(e.target.value as 'dark' | 'light')
              }
            >
              <FormControlLabel
                value="dark"
                control={
                  <Radio
                    size="small"
                    sx={{
                      color: 'rgba(255, 179, 0, 0.5)',
                      '&.Mui-checked': {
                        color: anafisTheme.colors.tabs.fitting.main,
                      },
                    }}
                  />
                }
                label={
                  <Typography
                    sx={{
                      fontSize: 13,
                      color: anafisTheme.colors.text.primary,
                    }}
                  >
                    Dark background
                  </Typography>
                }
              />
              <FormControlLabel
                value="light"
                control={
                  <Radio
                    size="small"
                    sx={{
                      color: 'rgba(255, 179, 0, 0.5)',
                      '&.Mui-checked': {
                        color: anafisTheme.colors.tabs.fitting.main,
                      },
                    }}
                  />
                }
                label={
                  <Typography
                    sx={{
                      fontSize: 13,
                      color: anafisTheme.colors.text.primary,
                    }}
                  >
                    Light background
                  </Typography>
                }
              />
            </RadioGroup>
          </FormControl>

          <Typography
            variant="caption"
            sx={{
              color: anafisTheme.colors.text.tertiary,
              fontSize: 10,
              fontWeight: 600,
              mb: 0.5,
              display: 'block',
            }}
          >
            CONTENT
          </Typography>
          <FormControlLabel
            control={
              <Checkbox
                size="small"
                checked={includeResidualsInExport}
                onChange={(event) =>
                  setIncludeResidualsInExport(event.target.checked)
                }
                disabled={!canIncludeResiduals || isExporting}
                sx={{
                  color: 'rgba(255, 179, 0, 0.5)',
                  '&.Mui-checked': {
                    color: anafisTheme.colors.tabs.fitting.main,
                  },
                }}
              />
            }
            label={
              <Typography
                sx={{
                  fontSize: 13,
                  color: anafisTheme.colors.text.primary,
                }}
              >
                Include residuals chart
              </Typography>
            }
            sx={{ ml: 0 }}
          />
          {!canIncludeResiduals && (
            <Typography
              variant="caption"
              sx={{
                color: anafisTheme.colors.text.tertiary,
                fontSize: 11,
                display: 'block',
                ml: 0.5,
              }}
            >
              Residuals are available after a successful fit.
            </Typography>
          )}
        </DialogContent>
        <DialogActions
          sx={{
            borderTop: `1px solid ${anafisTheme.colors.border.light}`,
            p: 2,
          }}
        >
          <Button
            onClick={() => setExportDialogOpen(false)}
            disabled={isExporting}
            sx={{
              color: anafisTheme.colors.text.secondary,
              textTransform: 'none',
            }}
          >
            Cancel
          </Button>
          <Button
            onClick={() => void handleExport()}
            disabled={isExporting}
            variant="contained"
            sx={{
              bgcolor: 'rgba(255, 179, 0, 0.15)',
              color: anafisTheme.colors.tabs.fitting.main,
              '&:hover': { bgcolor: 'rgba(255, 179, 0, 0.25)' },
              textTransform: 'none',
              boxShadow: 'none',
            }}
          >
            {isExporting ? 'Exporting...' : 'Export'}
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}
