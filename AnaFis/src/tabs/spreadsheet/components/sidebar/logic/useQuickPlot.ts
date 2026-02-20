/**
 * useQuickPlot hook - Extracted business logic for quick plotting operations
 *
 * This hook encapsulates all the business logic for creating quick plots from spreadsheet data,
 * including state management, validation, data extraction, chart generation, and export functionality.
 *
 * Uses Plotly.js for rendering — chart options are returned as Plotly data/layout
 * and the consuming component renders via <Plot />.
 */

import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import { useCallback, useMemo, useRef, useState } from 'react';
import { Plotly } from '@/shared/components/PlotlyChart';
import { CHART_COLORS, getThemeLayout } from '@/shared/components/plotlyTheme';
import type { SpreadsheetRef } from '@/tabs/spreadsheet/types/SpreadsheetInterface';

interface UseQuickPlotOptions {
  spreadsheetRef: React.RefObject<SpreadsheetRef | null>;
  onSelectionChange?: (selection: string) => void;
}

type PlotType = 'scatter' | 'line' | 'both';
type ExportFormat = 'png' | 'svg';
type ExportTheme = 'dark' | 'light';

export function useQuickPlot({
  spreadsheetRef,
  onSelectionChange,
}: UseQuickPlotOptions) {
  // Quick plot configuration state
  const [xRange, setXRange] = useState<string>('');
  const [yRange, setYRange] = useState<string>('');
  const [errorRange, setErrorRange] = useState<string>('');
  const [xLabel, setXLabel] = useState<string>('');
  const [yLabel, setYLabel] = useState<string>('');
  const [plotType, setPlotType] = useState<PlotType>('scatter');
  const [showErrorBars, setShowErrorBars] = useState<boolean>(false);

  // Plot data and state
  const [xData, setXData] = useState<number[]>([]);
  const [yData, setYData] = useState<number[]>([]);
  const [errorData, setErrorData] = useState<number[] | undefined>(undefined);
  const [isGenerating, setIsGenerating] = useState<boolean>(false);
  const [error, setError] = useState<string>('');
  const [hasPlot, setHasPlot] = useState<boolean>(false);

  // Chart ref — Plotly manages internals, we just need the div for export
  const chartRef = useRef<HTMLDivElement>(null);
  const chartReady = true; // Plotly is always ready

  // Export state
  const [exportDialogOpen, setExportDialogOpen] = useState(false);
  const [exportTheme, setExportTheme] = useState<ExportTheme>('dark');
  const [exportFormat, setExportFormat] = useState<ExportFormat>('png');
  const [isExporting, setIsExporting] = useState(false);

  // Save to library state
  const [saveDialogOpen, setSaveDialogOpen] = useState(false);
  const [xSequenceName, setXSequenceName] = useState('');
  const [ySequenceName, setYSequenceName] = useState('');
  const [xUnit, setXUnit] = useState('');
  const [yUnit, setYUnit] = useState('');
  const [sequenceTags, setSequenceTags] = useState<string[]>(['quick_plot']);
  const [newTag, setNewTag] = useState('');

  // Validate plot configuration
  const validateConfig = useCallback(async (): Promise<boolean> => {
    if (!xRange.trim()) {
      setError('X range is required');
      return false;
    }
    if (!yRange.trim()) {
      setError('Y range is required');
      return false;
    }

    const spreadsheetAPI = spreadsheetRef.current;
    if (!spreadsheetAPI) {
      setError('Spreadsheet not initialized');
      return false;
    }

    try {
      // Get X data
      const xValues = await spreadsheetAPI.getRange(xRange);
      const xFlat = xValues
        .flat()
        .map((v: unknown) => {
          const num = typeof v === 'number' ? v : parseFloat(String(v));
          return Math.round(num * 1e10) / 1e10;
        })
        .filter((num) => Number.isFinite(num));

      // Get Y data
      const yValues = await spreadsheetAPI.getRange(yRange);
      const yFlat = yValues
        .flat()
        .map((v: unknown) => {
          const num = typeof v === 'number' ? v : parseFloat(String(v));
          return Math.round(num * 1e10) / 1e10;
        })
        .filter((num) => Number.isFinite(num));

      // Validate lengths
      if (xFlat.length !== yFlat.length) {
        setError(
          `Length mismatch: X has ${xFlat.length} points, Y has ${yFlat.length} points`
        );
        return false;
      }

      if (xFlat.length === 0) {
        setError('No data found in ranges');
        return false;
      }

      // Get error bars if enabled
      let errorFlat: number[] | undefined;
      if (showErrorBars && errorRange) {
        const errorValues = await spreadsheetAPI.getRange(errorRange);
        errorFlat = errorValues
          .flat()
          .map((v: unknown) => {
            const num = typeof v === 'number' ? v : parseFloat(String(v));
            return Math.round(Math.abs(num) * 1e10) / 1e10;
          })
          .filter((num) => Number.isFinite(num));

        if (errorFlat.length !== yFlat.length) {
          setError(
            `Error range length (${errorFlat.length}) doesn't match Y data (${yFlat.length})`
          );
          return false;
        }
      }

      // All validation passed
      setXData(xFlat);
      setYData(yFlat);
      setErrorData(errorFlat);
      setHasPlot(true);
      return true;
    } catch (err) {
      setError(
        `Range validation failed: ${err instanceof Error ? err.message : String(err)}`
      );
      return false;
    }
  }, [xRange, yRange, errorRange, showErrorBars, spreadsheetRef]);

  // Generate Plotly data and layout
  const generatePlotlyProps = useCallback(
    (
      theme: 'dark' | 'light' = 'dark'
    ): { data: Plotly.Data[]; layout: Partial<Plotly.Layout> } => {
      const { layout: baseLayout, axis } = getThemeLayout(theme);
      const traces: Plotly.Data[] = [];

      // Scatter series
      if (plotType === 'scatter' || plotType === 'both') {
        const scatter: Plotly.Data = {
          x: xData,
          y: yData,
          mode: 'markers',
          type: 'scatter',
          name: yLabel || 'Data',
          marker: { color: CHART_COLORS.primary, size: 6 },
        };

        if (showErrorBars && errorData) {
          scatter.error_y = {
            type: 'data',
            array: errorData,
            visible: true,
            color: CHART_COLORS.error,
            thickness: 1.5,
            width: 3,
          };
        }

        traces.push(scatter);
      }

      // Line series
      if (plotType === 'line' || plotType === 'both') {
        traces.push({
          x: xData,
          y: yData,
          mode: 'lines',
          type: 'scatter',
          name: plotType === 'both' ? 'Trend' : yLabel || 'Data',
          line: {
            color:
              plotType === 'both'
                ? CHART_COLORS.secondary
                : CHART_COLORS.primary,
            width: 2,
          },
        });
      }

      // Calculate axis ranges with 10% margin
      const xMin = Math.min(...xData);
      const xMax = Math.max(...xData);
      const xSpan = xMax - xMin;
      const xMargin = Math.max(xSpan * 0.1, Math.abs(xMin) * 0.01, 0.1);

      let yMin: number, yMax: number;
      if (showErrorBars && errorData) {
        const yWithErrorsMin = yData.map((y, i) => y - (errorData[i] ?? 0));
        const yWithErrorsMax = yData.map((y, i) => y + (errorData[i] ?? 0));
        yMin = Math.min(...yWithErrorsMin);
        yMax = Math.max(...yWithErrorsMax);
      } else {
        yMin = Math.min(...yData);
        yMax = Math.max(...yData);
      }
      const ySpan = yMax - yMin;
      const yMargin = Math.max(ySpan * 0.1, Math.abs(yMin) * 0.01, 0.1);

      const layout: Partial<Plotly.Layout> = {
        ...baseLayout,
        showlegend: plotType === 'both',
        legend:
          plotType === 'both'
            ? {
                font: { color: baseLayout.font?.color as string, size: 10 },
                bgcolor: 'rgba(0,0,0,0.4)',
                borderwidth: 0,
                x: 0.01,
                y: 0.99,
                xanchor: 'left',
                yanchor: 'top',
              }
            : undefined,
        xaxis: {
          ...axis,
          title: { text: xLabel || 'X', font: { color: '#aaa', size: 12 } },
          range: [xMin - xMargin, xMax + xMargin],
        },
        yaxis: {
          ...axis,
          title: { text: yLabel || 'Y', font: { color: '#aaa', size: 12 } },
          range: [yMin - yMargin, yMax + yMargin],
        },
      };

      return { data: traces, layout };
    },
    [xData, yData, errorData, plotType, xLabel, yLabel, showErrorBars]
  );

  // Chart data for the consuming component
  const plotlyData = useMemo(() => {
    if (!hasPlot || xData.length === 0 || yData.length === 0) return null;
    return generatePlotlyProps('dark');
  }, [hasPlot, xData, yData, generatePlotlyProps]);

  // Generate plot
  const generatePlot = useCallback(async (): Promise<void> => {
    setError('');

    if (!spreadsheetRef.current) {
      setError('Spreadsheet not initialized');
      return;
    }

    setIsGenerating(true);
    try {
      await validateConfig();
    } finally {
      setIsGenerating(false);
    }
  }, [spreadsheetRef, validateConfig]);

  // Export functionality
  const handleExport = useCallback(async () => {
    if (!hasPlot) {
      return;
    }

    setIsExporting(true);

    try {
      const extension = exportFormat === 'svg' ? 'svg' : 'png';
      const filterName = exportFormat === 'svg' ? 'SVG Image' : 'PNG Image';

      const filePath = await save({
        defaultPath: `quick_plot_${Date.now()}.${extension}`,
        filters: [
          {
            name: filterName,
            extensions: [extension],
          },
        ],
      });

      if (!filePath) {
        setExportDialogOpen(false);
        setIsExporting(false);
        return;
      }

      // Generate export props with the chosen theme
      const { data, layout } = generatePlotlyProps(exportTheme);

      // Create a temporary offscreen div for Plotly rendering
      const tempDiv = document.createElement('div');
      tempDiv.style.width = '1200px';
      tempDiv.style.height = '800px';
      tempDiv.style.position = 'absolute';
      tempDiv.style.left = '-9999px';
      document.body.appendChild(tempDiv);

      try {
        // Render Plotly chart in offscreen div
        await Plotly.newPlot(tempDiv, data, {
          ...layout,
          width: 1200,
          height: 800,
        });

        if (exportFormat === 'svg') {
          const svgDataUrl = await Plotly.toImage(tempDiv, {
            format: 'svg',
            width: 1200,
            height: 800,
          });

          // Plotly returns either:
          //   "data:image/svg+xml,<url-encoded-svg>"   (plotly.js-dist-min)
          //   "data:image/svg+xml;base64,<b64>"         (full plotly.js)
          let svgString: string;
          if (svgDataUrl.includes(';base64,')) {
            const base64 = svgDataUrl.split(';base64,')[1] ?? '';
            svgString = atob(base64);
          } else {
            // URL-encoded form: "data:image/svg+xml,..."
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
      setIsExporting(false);
    } catch (error) {
      setError(`Export failed: ${String(error)}`);
      setExportDialogOpen(false);
      setIsExporting(false);
    }
  }, [hasPlot, exportFormat, exportTheme, generatePlotlyProps]);

  // Save to library functionality
  const handleSaveToLibrary = useCallback(async () => {
    if (!xSequenceName || !ySequenceName) {
      setError('Please provide names for both sequences');
      return;
    }

    try {
      // Save X sequence
      await invoke('save_sequence', {
        request: {
          name: xSequenceName,
          description: `X-axis data from ${xRange}`,
          tags: [...sequenceTags, 'x_axis'],
          unit: xUnit,
          source: `Quick Plot: ${String(xRange)}`,
          data: xData,
          uncertainties: null,
          is_pinned: false,
        },
      });

      // Save Y sequence
      await invoke('save_sequence', {
        request: {
          name: ySequenceName,
          description: `Y-axis data from ${yRange}`,
          tags: [...sequenceTags, 'y_axis'],
          unit: yUnit,
          source: `Quick Plot: ${String(yRange)}`,
          data: yData,
          uncertainties: errorData ?? null,
          is_pinned: false,
        },
      });

      setSaveDialogOpen(false);
      setError('');
      setXSequenceName('');
      setYSequenceName('');
      setXUnit('');
      setYUnit('');
      setSequenceTags(['quick_plot']);
    } catch (error) {
      setError(`Failed to save: ${String(error)}`);
    }
  }, [
    xSequenceName,
    ySequenceName,
    xRange,
    yRange,
    xUnit,
    yUnit,
    sequenceTags,
    xData,
    yData,
    errorData,
  ]);

  // Tag management
  const addTag = useCallback(() => {
    if (newTag.trim() && !sequenceTags.includes(newTag.trim())) {
      setSequenceTags([...sequenceTags, newTag.trim()]);
      setNewTag('');
    }
  }, [newTag, sequenceTags]);

  const removeTag = useCallback(
    (tag: string) => {
      setSequenceTags(sequenceTags.filter((t) => t !== tag));
    },
    [sequenceTags]
  );

  // Handle selection change for ranges
  const handleSelectionChange = useCallback(
    (selection: string) => {
      onSelectionChange?.(selection);
    },
    [onSelectionChange]
  );

  // Clear error
  const clearError = useCallback(() => {
    setError('');
  }, []);

  return {
    // Configuration state
    xRange,
    yRange,
    errorRange,
    xLabel,
    yLabel,
    plotType,
    showErrorBars,

    // Data and plot state
    xData,
    yData,
    errorData,
    isGenerating,
    error,
    hasPlot,
    chartRef,
    chartReady,

    // Plotly data/layout for consuming component
    plotlyData,

    // Export state
    exportDialogOpen,
    exportTheme,
    exportFormat,
    isExporting,

    // Save state
    saveDialogOpen,
    xSequenceName,
    ySequenceName,
    xUnit,
    yUnit,
    sequenceTags,
    newTag,

    // Actions
    setXRange,
    setYRange,
    setErrorRange,
    setXLabel,
    setYLabel,
    setPlotType,
    setShowErrorBars,
    generatePlot,
    handleSelectionChange,
    clearError,

    // Export actions
    setExportDialogOpen,
    setExportTheme,
    setExportFormat,
    handleExport,

    // Save actions
    setSaveDialogOpen,
    setXSequenceName,
    setYSequenceName,
    setXUnit,
    setYUnit,
    handleSaveToLibrary,
    addTag,
    removeTag,
    setNewTag,
  };
}
