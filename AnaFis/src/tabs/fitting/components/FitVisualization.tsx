import ImageIcon from '@mui/icons-material/Image';
import { Box, Button, CircularProgress, Dialog, DialogActions, DialogContent, DialogTitle, FormControl, FormControlLabel, IconButton, Radio, RadioGroup, Tooltip, Typography } from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import Plot, { Plotly } from '@/shared/components/PlotlyChart';
import { ANAFIS_CHART_CONFIG } from '@/shared/components/plotlyTheme';

import type {
  AxisSettings,
  DependentBinding,
  GridEvaluationResponse,
  ImportedData,
  OdrFitResponse,
  VariableBinding,
} from '../types/fittingTypes';
import { anafisTheme } from '@/shared/theme/unifiedTheme';
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

  const [exportDialogOpen, setExportDialogOpen] = useState(false);
  const [exportFormat, setExportFormat] = useState<'png' | 'svg'>('png');
  const [exportTheme, setExportTheme] = useState<'dark' | 'light'>('dark');
  const [isExporting, setIsExporting] = useState(false);

  const handleExport = useCallback(async () => {
    if (mode === 'empty') return;
    setIsExporting(true);
    try {
      const extension = exportFormat;
      const filterName = exportFormat === 'svg' ? 'SVG Image' : 'PNG Image';

      const filePath = await save({
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
        const result = build2DChart(importedData!, variableBindings, dependentBinding, axisSettings, fitResult, exportTheme);
        exportData = result.data;
        exportLayout = result.layout;
      } else if (mode === '3d') {
        const result = build3DChart(importedData!, variableBindings, dependentBinding, axisSettings, fitResult, gridData, exportTheme);
        exportData = result.data;
        exportLayout = result.layout;
      } else if (mode === 'predicted') {
        const result = buildPredictedChart(importedData!, dependentBinding, axisSettings, fitResult, exportTheme);
        exportData = result.data;
        exportLayout = result.layout;
      } else {
        exportData = data;
        exportLayout = layout;
      }

      const tempDiv = document.createElement('div');
      tempDiv.style.width = '1200px';
      tempDiv.style.height = '800px';
      tempDiv.style.position = 'absolute';
      tempDiv.style.left = '-9999px';
      document.body.appendChild(tempDiv);

      try {
        await Plotly.newPlot(tempDiv, exportData, {
          ...exportLayout,
          width: 1200,
          height: 800,
        });

        if (exportFormat === 'svg') {
          const svgDataUrl = await Plotly.toImage(tempDiv, {
            format: 'svg',
            width: 1200,
            height: 800,
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
            width: 1200,
            height: 800,
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
    importedData,
    variableBindings,
    dependentBinding,
    axisSettings,
    fitResult,
    gridData,
    data,
    layout
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
              {isExporting ? <CircularProgress size={16} /> : <ImageIcon fontSize="small" />}
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
          paper: { sx: { bgcolor: anafisTheme.colors.background.elevated, backgroundImage: 'none', color: anafisTheme.colors.text.primary } },
        }}
      >
        <DialogTitle sx={{ color: anafisTheme.colors.tabs.fitting.main, borderBottom: `1px solid ${anafisTheme.colors.border.light}` }}>
          Export Plot
        </DialogTitle>
        <DialogContent sx={{ pt: 3 }}>
          <Typography variant="body2" sx={{ color: anafisTheme.colors.text.secondary, mb: 2 }}>
            Choose export format and theme:
          </Typography>

          <Typography variant="caption" sx={{ color: anafisTheme.colors.text.tertiary, fontSize: 10, fontWeight: 600, mb: 0.5, display: 'block' }}>
            FORMAT
          </Typography>
          <FormControl sx={{ mb: 2 }}>
            <RadioGroup value={exportFormat} onChange={(e) => setExportFormat(e.target.value as 'png' | 'svg')}>
              <FormControlLabel value="png" control={<Radio size="small" sx={{ color: 'rgba(255, 179, 0, 0.5)', '&.Mui-checked': { color: anafisTheme.colors.tabs.fitting.main } }} />} label={<Typography sx={{ fontSize: 13, color: anafisTheme.colors.text.primary }}>PNG (Raster)</Typography>} />
              <FormControlLabel value="svg" control={<Radio size="small" sx={{ color: 'rgba(255, 179, 0, 0.5)', '&.Mui-checked': { color: anafisTheme.colors.tabs.fitting.main } }} />} label={<Typography sx={{ fontSize: 13, color: anafisTheme.colors.text.primary }}>SVG (Vector)</Typography>} />
            </RadioGroup>
          </FormControl>

          <Typography variant="caption" sx={{ color: anafisTheme.colors.text.tertiary, fontSize: 10, fontWeight: 600, mb: 0.5, display: 'block' }}>
            THEME
          </Typography>
          <FormControl>
            <RadioGroup value={exportTheme} onChange={(e) => setExportTheme(e.target.value as 'dark' | 'light')}>
              <FormControlLabel value="dark" control={<Radio size="small" sx={{ color: 'rgba(255, 179, 0, 0.5)', '&.Mui-checked': { color: anafisTheme.colors.tabs.fitting.main } }} />} label={<Typography sx={{ fontSize: 13, color: anafisTheme.colors.text.primary }}>Dark background</Typography>} />
              <FormControlLabel value="light" control={<Radio size="small" sx={{ color: 'rgba(255, 179, 0, 0.5)', '&.Mui-checked': { color: anafisTheme.colors.tabs.fitting.main } }} />} label={<Typography sx={{ fontSize: 13, color: anafisTheme.colors.text.primary }}>Light background</Typography>} />
            </RadioGroup>
          </FormControl>
        </DialogContent>
        <DialogActions sx={{ borderTop: `1px solid ${anafisTheme.colors.border.light}`, p: 2 }}>
          <Button onClick={() => setExportDialogOpen(false)} disabled={isExporting} sx={{ color: anafisTheme.colors.text.secondary, textTransform: 'none' }}>
            Cancel
          </Button>
          <Button onClick={() => void handleExport()} disabled={isExporting} variant="contained" sx={{ bgcolor: 'rgba(255, 179, 0, 0.15)', color: anafisTheme.colors.tabs.fitting.main, '&:hover': { bgcolor: 'rgba(255, 179, 0, 0.25)' }, textTransform: 'none', boxShadow: 'none' }}>
            {isExporting ? 'Exporting...' : 'Export'}
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}
