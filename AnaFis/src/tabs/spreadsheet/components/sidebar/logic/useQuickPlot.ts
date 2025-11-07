/**
 * useQuickPlot hook - Extracted business logic for quick plotting operations
 *
 * This hook encapsulates all the business logic for creating quick plots from spreadsheet data,
 * including state management, validation, data extraction, chart generation, and export functionality.
 */

import { useState, useCallback, useRef, useEffect } from 'react';
import { SpreadsheetRef } from '@/tabs/spreadsheet/types/SpreadsheetInterface';
import * as echarts from 'echarts';
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';

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

  // Chart management
  const chartRef = useRef<HTMLDivElement>(null);
  const chartInstanceRef = useRef<echarts.ECharts | null>(null);
  const [chartReady, setChartReady] = useState<boolean>(false);

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
      const xFlat = xValues.flat().map((v: unknown) => {
        const num = typeof v === 'number' ? v : parseFloat(String(v));
        return Math.round(num * 1e10) / 1e10;
      }).filter(num => isFinite(num));

      // Get Y data
      const yValues = await spreadsheetAPI.getRange(yRange);
      const yFlat = yValues.flat().map((v: unknown) => {
        const num = typeof v === 'number' ? v : parseFloat(String(v));
        return Math.round(num * 1e10) / 1e10;
      }).filter(num => isFinite(num));

      // Validate lengths
      if (xFlat.length !== yFlat.length) {
        setError(`Length mismatch: X has ${xFlat.length} points, Y has ${yFlat.length} points`);
        return false;
      }

      if (xFlat.length === 0) {
        setError('No data found in ranges');
        return false;
      }

      // Get error bars if enabled
      let errorFlat: number[] | undefined = undefined;
      if (showErrorBars && errorRange) {
        const errorValues = await spreadsheetAPI.getRange(errorRange);
        errorFlat = errorValues.flat().map((v: unknown) => {
          const num = typeof v === 'number' ? v : parseFloat(String(v));
          return Math.round(Math.abs(num) * 1e10) / 1e10;
        }).filter(num => isFinite(num));

        if (errorFlat.length !== yFlat.length) {
          setError(`Error range length (${errorFlat.length}) doesn't match Y data (${yFlat.length})`);
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
      setError(`Range validation failed: ${err instanceof Error ? err.message : String(err)}`);
      return false;
    }
  }, [xRange, yRange, errorRange, showErrorBars, spreadsheetRef]);

  // Chart management functions
  const initializeChart = useCallback(() => {
    if (chartRef.current && !chartInstanceRef.current) {
      chartInstanceRef.current = echarts.init(chartRef.current, null, {
        renderer: 'canvas',
        devicePixelRatio: 2
      });
      setChartReady(true);
    }
  }, []);

  const generateChartOptions = useCallback((theme: 'dark' | 'light' = 'dark'): echarts.EChartsOption => {
    const series: echarts.SeriesOption[] = [];

    // Add scatter series
    if (plotType === 'scatter' || plotType === 'both') {
      const scatterData = xData.map((x, i) => [x, yData[i]]);

      series.push({
        name: yLabel || 'Data',
        type: 'scatter',
        data: scatterData,
        symbolSize: 8,
        itemStyle: { color: '#2196f3' },
        z: 2,
      });

      // Add error bars if enabled
      if (showErrorBars && errorData) {
        series.push({
          name: 'Error Bars',
          type: 'custom',
          renderItem: (_params: echarts.CustomSeriesRenderItemParams, api: echarts.CustomSeriesRenderItemAPI) => {
            const point = api.coord([api.value(0), api.value(1)]);
            const errorValue = api.value(2) as number;
            const yTop = api.coord([api.value(0), (api.value(1) as number) + errorValue]);
            const yBottom = api.coord([api.value(0), (api.value(1) as number) - errorValue]);

            if (typeof point[0] !== 'number' || typeof point[1] !== 'number' ||
              typeof yTop[1] !== 'number' || typeof yBottom[1] !== 'number') {
              return { type: 'group', children: [] };
            }
            return {
              type: 'group',
              children: [
                {
                  type: 'line',
                  shape: { x1: point[0], y1: yTop[1], x2: point[0], y2: yBottom[1] },
                  style: { stroke: '#f44336', lineWidth: 1.5 },
                },
                {
                  type: 'line',
                  shape: { x1: point[0] - 4, y1: yTop[1], x2: point[0] + 4, y2: yTop[1] },
                  style: { stroke: '#f44336', lineWidth: 1.5 },
                },
                {
                  type: 'line',
                  shape: { x1: point[0] - 4, y1: yBottom[1], x2: point[0] + 4, y2: yBottom[1] },
                  style: { stroke: '#f44336', lineWidth: 1.5 },
                },
              ],
            };
          },
          data: xData.map((x, i) => [x, yData[i], errorData[i]]),
          z: 1,
          silent: true,
        });
      }
    }

    // Add line series
    if (plotType === 'line' || plotType === 'both') {
      series.push({
        name: plotType === 'both' ? 'Trend' : yLabel || 'Data',
        type: 'line',
        data: xData.map((x, i) => [x, yData[i]]),
        smooth: false,
        showSymbol: false,
        lineStyle: {
          color: plotType === 'both' ? '#4caf50' : '#2196f3',
          width: 2
        },
        z: 0,
      });
    }

    // Theme-specific colors
    const textColor = theme === 'light' ? '#000000' : '#ffffff';
    const axisLineColor = theme === 'light' ? 'rgba(0,0,0,0.3)' : 'rgba(255,255,255,0.3)';
    const splitLineColor = theme === 'light' ? 'rgba(0,0,0,0.1)' : 'rgba(255,255,255,0.1)';
    const backgroundColor = theme === 'light' ? '#ffffff' : '#0a0a0a';

    // Calculate axis ranges with 10% margin
    const xMin = Math.min(...xData);
    const xMax = Math.max(...xData);
    const xRange = xMax - xMin;
    const xMargin = Math.max(xRange * 0.10, Math.abs(xMin) * 0.01, 0.1);

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
    const yRange = yMax - yMin;
    const yMargin = Math.max(yRange * 0.10, Math.abs(yMin) * 0.01, 0.1);

    const roundToPrecision = (num: number, precision: number = 10): number => {
      if (!isFinite(num)) { return 0; }
      return Math.round(num * Math.pow(10, precision)) / Math.pow(10, precision);
    };

    const option: echarts.EChartsOption = {
      backgroundColor,
      grid: {
        left: 60,
        right: 30,
        top: 40,
        bottom: 60,
        containLabel: false,
      },
      xAxis: {
        type: 'value',
        name: xLabel || 'X',
        nameLocation: 'middle',
        nameGap: 35,
        nameTextStyle: { color: textColor, fontSize: 12 },
        axisLine: { lineStyle: { color: axisLineColor } },
        axisLabel: {
          color: textColor,
          formatter: (value: number) => {
            if (Math.abs(value) >= 1e6 || (Math.abs(value) < 0.001 && value !== 0)) {
              return value.toExponential(2);
            } else if (Math.abs(value) >= 1000) {
              return value.toFixed(0);
            } else if (Math.abs(value) >= 1) {
              return value.toFixed(2);
            } else {
              return value.toFixed(4);
            }
          }
        },
        splitLine: { lineStyle: { color: splitLineColor } },
        min: roundToPrecision(xMin - xMargin),
        max: roundToPrecision(xMax + xMargin),
      },
      yAxis: {
        type: 'value',
        name: yLabel || 'Y',
        nameLocation: 'middle',
        nameGap: 45,
        nameTextStyle: { color: textColor, fontSize: 12 },
        axisLine: { lineStyle: { color: axisLineColor } },
        axisLabel: {
          color: textColor,
          formatter: (value: number) => {
            if (Math.abs(value) >= 1e6 || (Math.abs(value) < 0.001 && value !== 0)) {
              return value.toExponential(2);
            } else if (Math.abs(value) >= 1000) {
              return value.toFixed(0);
            } else if (Math.abs(value) >= 1) {
              return value.toFixed(2);
            } else {
              return value.toFixed(4);
            }
          }
        },
        splitLine: { lineStyle: { color: splitLineColor } },
        min: roundToPrecision(yMin - yMargin),
        max: roundToPrecision(yMax + yMargin),
      },
      series,
      legend: plotType === 'both' ? {
        show: true,
        textStyle: { color: textColor },
        top: 5,
        backgroundColor: theme === 'light' ? 'rgba(255,255,255,0.8)' : 'rgba(0,0,0,0.5)',
        borderRadius: 4,
        padding: 8,
      } : { show: false },
      tooltip: {
        trigger: 'axis',
        axisPointer: { type: 'cross' },
        backgroundColor: theme === 'light' ? 'rgba(255,255,255,0.95)' : 'rgba(0,0,0,0.8)',
        borderColor: '#2196f3',
        textStyle: { color: textColor },
      },
    };

    return option;
  }, [xData, yData, errorData, plotType, xLabel, yLabel, showErrorBars]);

  const updateChart = useCallback(() => {
    if (!chartInstanceRef.current || !hasPlot || xData.length === 0 || yData.length === 0) {
      return;
    }

    const option = generateChartOptions('dark');
    chartInstanceRef.current.setOption(option, true);
  }, [hasPlot, xData, yData, generateChartOptions]);

  // Generate plot
  const generatePlot = useCallback(async (): Promise<void> => {
    setError('');

    if (!spreadsheetRef.current) {
      setError('Spreadsheet not initialized');
      return;
    }

    setIsGenerating(true);
    try {
      const success = await validateConfig();
      if (success) {
        updateChart();
      }
    } finally {
      setIsGenerating(false);
    }
  }, [spreadsheetRef, validateConfig, updateChart]);

  // Export functionality
  const handleExport = useCallback(async () => {
    if (!hasPlot || !chartInstanceRef.current) { return; }

    setIsExporting(true);

    try {
      const extension = exportFormat === 'svg' ? 'svg' : 'png';
      const filterName = exportFormat === 'svg' ? 'SVG Image' : 'PNG Image';

      const filePath = await save({
        defaultPath: `quick_plot_${Date.now()}.${extension}`,
        filters: [{
          name: filterName,
          extensions: [extension]
        }]
      });

      if (!filePath) {
        setExportDialogOpen(false);
        setIsExporting(false);
        return;
      }

      if (exportFormat === 'svg') {
        const tempDiv = document.createElement('div');
        tempDiv.style.width = '1200px';
        tempDiv.style.height = '800px';
        tempDiv.style.position = 'absolute';
        tempDiv.style.left = '-9999px';
        document.body.appendChild(tempDiv);

        const tempChart = echarts.init(tempDiv, null, { renderer: 'svg' });
        const exportOptions = generateChartOptions(exportTheme);
        exportOptions.animation = false;
        tempChart.setOption(exportOptions);

        await new Promise<void>((resolve) => {
          tempChart.on('finished', () => resolve());
          setTimeout(resolve, 500);
        });

        const svgString = tempChart.renderToSVGString();
        tempChart.dispose();
        document.body.removeChild(tempDiv);

        await invoke('save_svg_file', {
          svgContent: svgString,
          path: filePath,
        });
      } else {
        const tempDiv = document.createElement('div');
        tempDiv.style.width = '1200px';
        tempDiv.style.height = '800px';
        tempDiv.style.position = 'absolute';
        tempDiv.style.left = '-9999px';
        document.body.appendChild(tempDiv);

        const tempChart = echarts.init(tempDiv, null, { renderer: 'canvas' });
        const exportOptions = generateChartOptions(exportTheme);
        exportOptions.animation = false;
        tempChart.setOption(exportOptions);

        await new Promise<void>((resolve) => {
          tempChart.on('finished', () => resolve());
          setTimeout(resolve, 500);
        });

        const dataURL = tempChart.getDataURL({
          type: 'png',
          pixelRatio: 2,
          backgroundColor: exportTheme === 'light' ? '#ffffff' : '#0a0a0a',
        });

        tempChart.dispose();
        document.body.removeChild(tempDiv);

        await invoke('save_image_from_data_url', {
          dataUrl: dataURL,
          path: filePath,
        });
      }

      setExportDialogOpen(false);
      setIsExporting(false);
    } catch (error) {
      setError(`Export failed: ${String(error)}`);
      setExportDialogOpen(false);
      setIsExporting(false);
    }
  }, [hasPlot, exportFormat, exportTheme, generateChartOptions]);

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
  }, [xSequenceName, ySequenceName, xRange, yRange, xUnit, yUnit, sequenceTags, xData, yData, errorData]);

  // Tag management
  const addTag = useCallback(() => {
    if (newTag.trim() && !sequenceTags.includes(newTag.trim())) {
      setSequenceTags([...sequenceTags, newTag.trim()]);
      setNewTag('');
    }
  }, [newTag, sequenceTags]);

  const removeTag = useCallback((tag: string) => {
    setSequenceTags(sequenceTags.filter(t => t !== tag));
  }, [sequenceTags]);

  // Initialize chart when component mounts
  useEffect(() => {
    if (hasPlot) {
      initializeChart();
    }
  }, [hasPlot, initializeChart]);

  // Update chart when data changes
  useEffect(() => {
    if (hasPlot && chartReady) {
      updateChart();
    }
  }, [hasPlot, chartReady, updateChart]);

  // Handle selection change for ranges
  const handleSelectionChange = useCallback((selection: string) => {
    onSelectionChange?.(selection);
  }, [onSelectionChange]);

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
  }
} 