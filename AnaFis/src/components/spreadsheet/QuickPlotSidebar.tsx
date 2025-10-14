import React, { useState, useEffect, useCallback, useRef } from 'react';
import {
  Box,
  Typography,
  TextField,
  Button,
  IconButton,
  Alert,
  Checkbox,
  FormControlLabel,
  Radio,
  RadioGroup,
  FormControl,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Chip,
  CircularProgress,
} from '@mui/material';
import CloseIcon from '@mui/icons-material/Close';
import SaveIcon from '@mui/icons-material/Save';
import ImageIcon from '@mui/icons-material/Image';

import ShowChartIcon from '@mui/icons-material/ShowChart';
import * as echarts from 'echarts';
import { SpreadsheetRef } from './SpreadsheetInterface';
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import { sidebarStyles } from '../../utils/sidebarStyles';
import SidebarCard from '../ui/SidebarCard';
import { useSpreadsheetSelection } from '../../hooks/useSpreadsheetSelection';
import { anafisColors } from '../../themes';

// Icon aliases for clarity
const PlotIcon = ShowChartIcon;
const ExportIcon = ImageIcon;

interface QuickPlotSidebarProps {
  open: boolean;
  onClose: () => void;
  univerRef?: React.RefObject<SpreadsheetRef | null>;
  onSelectionChange?: (selection: string) => void;
  // Lifted state for persistence
  xRange: string;
  setXRange: (range: string) => void;
  yRange: string;
  setYRange: (range: string) => void;
  errorRange: string;
  setErrorRange: (range: string) => void;
  xLabel: string;
  setXLabel: (label: string) => void;
  yLabel: string;
  setYLabel: (label: string) => void;
  plotType: 'scatter' | 'line' | 'both';
  setPlotType: (type: 'scatter' | 'line' | 'both') => void;
  showErrorBars: boolean;
  setShowErrorBars: (show: boolean) => void;
}

type FocusedInputType = 'xRange' | 'yRange' | 'errorRange' | null;

const QuickPlotSidebar: React.FC<QuickPlotSidebarProps> = ({
  open,
  onClose,
  univerRef,
  onSelectionChange,
  xRange,
  setXRange,
  yRange,
  setYRange,
  errorRange,
  setErrorRange,
  xLabel,
  setXLabel,
  yLabel,
  setYLabel,
  plotType,
  setPlotType,
  showErrorBars,
  setShowErrorBars,
}) => {
  // ECharts refs
  const chartRef = useRef<HTMLDivElement>(null);
  const chartInstanceRef = useRef<echarts.ECharts | null>(null);

  // State for plot data
  const [xData, setXData] = useState<number[]>([]);
  const [yData, setYData] = useState<number[]>([]);
  const [errorData, setErrorData] = useState<number[] | undefined>(undefined);
  const [validationError, setValidationError] = useState<string | null>(null);
  const [isPlotting, setIsPlotting] = useState(false);
  const [hasPlot, setHasPlot] = useState(false);
  const [chartReady, setChartReady] = useState(false);
  
  // Save to library dialog state
  const [saveDialogOpen, setSaveDialogOpen] = useState(false);
  const [xSequenceName, setXSequenceName] = useState('');
  const [ySequenceName, setYSequenceName] = useState('');
  const [xUnit, setXUnit] = useState('');
  const [yUnit, setYUnit] = useState('');
  const [sequenceTags, setSequenceTags] = useState<string[]>(['quick_plot']);
  const [newTag, setNewTag] = useState('');
  
  // Export PNG dialog state
  const [exportDialogOpen, setExportDialogOpen] = useState(false);
  const [exportTheme, setExportTheme] = useState<'dark' | 'light'>('dark');
  const [exportFormat, setExportFormat] = useState<'png' | 'svg'>('png');
  const [isExporting, setIsExporting] = useState(false);
  
  // Use the spreadsheet selection hook
  const { focusedInput, handleInputFocus, handleInputBlur } = useSpreadsheetSelection<FocusedInputType>({
    onSelectionChange,
    updateField: (inputType, selection) => {
      switch (inputType) {
        case 'xRange':
          setXRange(selection);
          break;
        case 'yRange':
          setYRange(selection);
          break;
        case 'errorRange':
          setErrorRange(selection);
          break;
      }
    },
    sidebarDataAttribute: 'data-quick-plot-sidebar',
    handlerName: '__quickPlotSelectionHandler',
  });

  // Initialize ECharts instance for sidebar plot
  useEffect(() => {
    // Only initialize when sidebar is open and chart should be visible
    if (open && hasPlot && chartRef.current && !chartInstanceRef.current) {
      // Small delay to ensure DOM is fully rendered with dimensions
      const timer = setTimeout(() => {
        if (chartRef.current && !chartInstanceRef.current) {
          chartInstanceRef.current = echarts.init(chartRef.current, null, { 
            renderer: 'canvas',
            devicePixelRatio: 2 
          });
          setChartReady(true);
        }
      }, 50);
      
      return () => clearTimeout(timer);
    }
    
    // Cleanup on unmount or when sidebar closes
    if (!open && chartInstanceRef.current) {
      chartInstanceRef.current.dispose();
      chartInstanceRef.current = null;
      setChartReady(false);
    }
  }, [open, hasPlot]);

  // Resize charts on window resize
  useEffect(() => {
    const handleResize = () => {
      chartInstanceRef.current?.resize();
    };
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  // Extract data from spreadsheet and validate
  const extractAndValidateData = async () => {
    if (!univerRef?.current || !xRange || !yRange) {
      setValidationError('Please specify both X and Y ranges');
      return false;
    }

    try {
      setIsPlotting(true);
      setValidationError(null);

      // Get X data
      const xValues = await univerRef.current.getRange(xRange);
      const xFlat = xValues.flat().map((v: unknown) => {
        const num = typeof v === 'number' ? v : parseFloat(String(v));
        return num;
      });

      // Get Y data
      const yValues = await univerRef.current.getRange(yRange);
      const yFlat = yValues.flat().map((v: unknown) => {
        const num = typeof v === 'number' ? v : parseFloat(String(v));
        return num;
      });

      // Validate lengths
      if (xFlat.length !== yFlat.length) {
        setValidationError(`Length mismatch: X has ${xFlat.length} points, Y has ${yFlat.length} points`);
        return false;
      }

      if (xFlat.length === 0) {
        setValidationError('No data found in ranges');
        return false;
      }

      // Check for invalid values
      if (xFlat.some((v: number) => !isFinite(v))) {
        setValidationError('X data contains invalid values (NaN or Infinity)');
        return false;
      }

      if (yFlat.some((v: number) => !isFinite(v))) {
        setValidationError('Y data contains invalid values (NaN or Infinity)');
        return false;
      }

      // Get error bars if enabled
      let errorFlat: number[] | undefined = undefined;
      if (showErrorBars && errorRange) {
        const errorValues = await univerRef.current.getRange(errorRange);
        errorFlat = errorValues.flat().map((v: unknown) => {
          const num = typeof v === 'number' ? v : parseFloat(String(v));
          return Math.abs(num);
        });

        if (errorFlat && errorFlat.length !== yFlat.length) {
          setValidationError(`Error range length (${errorFlat.length}) doesn't match Y data (${yFlat.length})`);
          return false;
        }

        if (errorFlat && errorFlat.some((v: number) => !isFinite(v))) {
          setValidationError('Error data contains invalid values');
          return false;
        }
      }

      // All validation passed
      setXData(xFlat);
      setYData(yFlat);
      setErrorData(errorFlat);
      setHasPlot(true);
      return true;
    } catch (error) {
      console.error('Failed to extract data:', error);
      setValidationError(`Error: ${error}`);
      return false;
    } finally {
      setIsPlotting(false);
    }
  };

  const handleUpdatePlot = async () => {
    const success = await extractAndValidateData();
    if (success) {
      updateChart(chartInstanceRef.current);
    }
  };

  // Helper function to generate chart options with optional theme override
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
        itemStyle: { color: anafisColors.spreadsheet },
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
          color: plotType === 'both' ? '#4caf50' : anafisColors.spreadsheet,
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
    const tooltipBg = theme === 'light' ? 'rgba(255,255,255,0.95)' : 'rgba(0,0,0,0.8)';
    const legendBg = theme === 'light' ? 'rgba(255,255,255,0.8)' : 'rgba(0,0,0,0.5)';

    // Calculate axis ranges with 5% margin
    const xMin = Math.min(...xData);
    const xMax = Math.max(...xData);
    const xRange = xMax - xMin;
    const xMargin = xRange * 0.10;
    
    const yMin = Math.min(...yData);
    const yMax = Math.max(...yData);
    const yRange = yMax - yMin;
    const yMargin = yRange * 0.10;

    const option: echarts.EChartsOption = {
      backgroundColor: backgroundColor,
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
        axisLabel: { color: textColor },
        splitLine: { lineStyle: { color: splitLineColor } },
        min: xMin - xMargin,
        max: xMax + xMargin,
      },
      yAxis: {
        type: 'value',
        name: yLabel || 'Y',
        nameLocation: 'middle',
        nameGap: 45,
        nameTextStyle: { color: textColor, fontSize: 12 },
        axisLine: { lineStyle: { color: axisLineColor } },
        axisLabel: { color: textColor },
        splitLine: { lineStyle: { color: splitLineColor } },
        min: yMin - yMargin,
        max: yMax + yMargin,
      },
      series,
      legend: plotType === 'both' ? {
        show: true,
        textStyle: { color: textColor },
        top: 5,
        backgroundColor: legendBg,
        borderRadius: 4,
        padding: 8,
      } : { show: false },
      tooltip: {
        trigger: 'axis',
        axisPointer: { type: 'cross' },
        backgroundColor: tooltipBg,
        borderColor: anafisColors.spreadsheet,
        textStyle: { color: textColor },
      },
    };

    return option;
  }, [xData, yData, errorData, plotType, xLabel, yLabel, showErrorBars]);

  // Update chart with current data
  const updateChart = useCallback((chartInstance: echarts.ECharts | null) => {
    if (!chartInstance || !hasPlot || xData.length === 0 || yData.length === 0) return;
    
    const option = generateChartOptions('dark');
    chartInstance.setOption(option, true);
  }, [hasPlot, xData, yData, generateChartOptions]);

  // Update chart when data changes
  useEffect(() => {
    if (hasPlot && chartReady) {
      updateChart(chartInstanceRef.current);
    }
  }, [hasPlot, chartReady, updateChart]);

  const handleSaveToLibrary = () => {
    // Pre-fill with labels if available
    if (!xSequenceName && xLabel) setXSequenceName(xLabel);
    if (!ySequenceName && yLabel) setYSequenceName(yLabel);
    setSaveDialogOpen(true);
  };

  const handleConfirmSave = async () => {
    if (!xSequenceName || !ySequenceName) {
      setValidationError('Please provide names for both sequences');
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
          source: `Quick Plot: ${xRange}`,
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
          source: `Quick Plot: ${yRange}`,
          data: yData,
          uncertainties: errorData || null,
          is_pinned: false,
        },
      });

      setSaveDialogOpen(false);
      setValidationError(null);
      // Clear the form for next use
      setXSequenceName('');
      setYSequenceName('');
      setXUnit('');
      setYUnit('');
      setSequenceTags(['quick_plot']);
    } catch (error) {
      console.error('Failed to save sequences:', error);
      setValidationError(`Failed to save: ${error}`);
    }
  };

  const handleExportPNG = () => {
    setExportDialogOpen(true);
  };

  const handleConfirmExport = async () => {
    if (!hasPlot || !chartInstanceRef.current) return;
    
    setIsExporting(true);
    
    try {
      // Determine file extension and filter based on format
      const extension = exportFormat === 'svg' ? 'svg' : 'png';
      const filterName = exportFormat === 'svg' ? 'SVG Image' : 'PNG Image';
      
      // Ask user where to save the file
      const filePath = await save({
        defaultPath: `quick_plot_${Date.now()}.${extension}`,
        filters: [{
          name: filterName,
          extensions: [extension]
        }]
      });
      
      if (!filePath) {
        // User cancelled
        setExportDialogOpen(false);
        setIsExporting(false);
        return;
      }
      
      if (exportFormat === 'svg') {
        // Export as SVG
        // Create a temporary chart with SVG renderer
        const tempDiv = document.createElement('div');
        tempDiv.style.width = '1200px';
        tempDiv.style.height = '800px';
        tempDiv.style.position = 'absolute';
        tempDiv.style.left = '-9999px';
        document.body.appendChild(tempDiv);
        
        const tempChart = echarts.init(tempDiv, null, { renderer: 'svg' });
        
        // Generate options with the selected theme
        const exportOptions = generateChartOptions(exportTheme);
        
        // Disable animation for immediate rendering
        exportOptions.animation = false;
        tempChart.setOption(exportOptions);
        
        // Wait for chart to be fully rendered using the 'finished' event
        await new Promise<void>((resolve) => {
          tempChart.on('finished', () => {
            resolve();
          });
          // Fallback timeout in case 'finished' doesn't fire
          setTimeout(resolve, 500);
        });
        
        // Get SVG string
        const svgString = tempChart.renderToSVGString();
        
        // Cleanup
        tempChart.dispose();
        document.body.removeChild(tempDiv);
        
        // Save SVG file
        await invoke('save_svg_file', {
          svgContent: svgString,
          path: filePath,
        });
      } else {
        // Export as PNG using data URL
        // Create a temporary chart with correct theme
        const tempDiv = document.createElement('div');
        tempDiv.style.width = '1200px';
        tempDiv.style.height = '800px';
        tempDiv.style.position = 'absolute';
        tempDiv.style.left = '-9999px';
        document.body.appendChild(tempDiv);
        
        const tempChart = echarts.init(tempDiv, null, { renderer: 'canvas' });
        
        // Generate options with the selected theme
        const exportOptions = generateChartOptions(exportTheme);
        
        // Disable animation for immediate rendering
        exportOptions.animation = false;
        tempChart.setOption(exportOptions);
        
        // Wait for chart to be fully rendered using the 'finished' event
        await new Promise<void>((resolve) => {
          tempChart.on('finished', () => {
            resolve();
          });
          // Fallback timeout in case 'finished' doesn't fire
          setTimeout(resolve, 500);
        });
        
        // Get PNG data URL
        const dataURL = tempChart.getDataURL({
          type: 'png',
          pixelRatio: 2,
          backgroundColor: exportTheme === 'light' ? '#ffffff' : '#0a0a0a',
        });
        
        // Cleanup
        tempChart.dispose();
        document.body.removeChild(tempDiv);
        
        // Save PNG file using data URL
        await invoke('save_image_from_data_url', {
          dataUrl: dataURL,
          path: filePath,
        });
      }
      
      setExportDialogOpen(false);
      setIsExporting(false);
    } catch (error) {
      console.error('Export failed:', error);
      setValidationError(`Export failed: ${error}`);
      setExportDialogOpen(false);
      setIsExporting(false);
    }
  };

  const handleAddTag = () => {
    if (newTag.trim() && !sequenceTags.includes(newTag.trim())) {
      setSequenceTags([...sequenceTags, newTag.trim()]);
      setNewTag('');
    }
  };

  const handleRemoveTag = (tag: string) => {
    setSequenceTags(sequenceTags.filter(t => t !== tag));
  };

  if (!open) return null;

  return (
    <Box sx={{ ...sidebarStyles.container, px: 1, pt: 2 }}>
      {/* Header */}
      <Box sx={sidebarStyles.header}>
        <Typography sx={sidebarStyles.text.header}>
          Quick Plot
        </Typography>
        <IconButton
          onClick={onClose}
          sx={sidebarStyles.iconButton}
        >
          <CloseIcon />
        </IconButton>
      </Box>

      {/* Main Content */}
      <Box sx={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden', gap: 2, p: 1.5 }}>
        {/* Data Input */}
        <SidebarCard title="Data Input" defaultExpanded={true}>
        {/* X-Axis Data */}
        <Box sx={{ mb: 2 }}>
          <Typography variant="caption" sx={{ color: 'rgba(255, 255, 255, 0.7)', fontSize: 10, fontWeight: 600, mb: 0.5, display: 'block' }}>
            X-AXIS DATA {focusedInput === 'xRange' && '← select range'}
          </Typography>
          <TextField
            fullWidth
            size="small"
            placeholder="e.g., A1:A100"
            value={xRange}
            onChange={(e) => setXRange(e.target.value)}
            onFocus={() => handleInputFocus('xRange')}
            onBlur={handleInputBlur}
            sx={{ mb: 1 }}
          />
          <TextField
            fullWidth
            size="small"
            placeholder="X-axis label"
            value={xLabel}
            onChange={(e) => setXLabel(e.target.value)}
          />
        </Box>

        {/* Y-Axis Data */}
        <Box sx={{ mb: 2 }}>
          <Typography variant="caption" sx={{ color: 'rgba(255, 255, 255, 0.7)', fontSize: 10, fontWeight: 600, mb: 0.5, display: 'block' }}>
            Y-AXIS DATA {focusedInput === 'yRange' && '← select range'}
          </Typography>
          <TextField
            fullWidth
            size="small"
            placeholder="e.g., B1:B100"
            value={yRange}
            onChange={(e) => setYRange(e.target.value)}
            onFocus={() => handleInputFocus('yRange')}
            onBlur={handleInputBlur}
            sx={{ mb: 1 }}
          />
          <TextField
            fullWidth
            size="small"
            placeholder="Y-axis label"
            value={yLabel}
            onChange={(e) => setYLabel(e.target.value)}
          />
        </Box>

        {/* Error Bars */}
        <Box sx={{ mb: 2 }}>
          <FormControlLabel
            control={
              <Checkbox
                checked={showErrorBars}
                onChange={(e) => setShowErrorBars(e.target.checked)}
                size="small"
                sx={{ color: 'rgba(33, 150, 243, 0.5)', '&.Mui-checked': { color: anafisColors.spreadsheet } }}
              />
            }
            label={
              <Typography sx={{ fontSize: 11, color: 'rgba(255, 255, 255, 0.9)' }}>
                Show error bars
              </Typography>
            }
          />
          {showErrorBars && (
            <>
              <Typography variant="caption" sx={{ color: 'rgba(255, 255, 255, 0.7)', fontSize: 10, fontWeight: 600, mb: 0.5, display: 'block', mt: 1 }}>
                ERROR BARS (±Y) {focusedInput === 'errorRange' && '← select range'}
              </Typography>
              <TextField
                fullWidth
                size="small"
                placeholder="e.g., C1:C100"
                value={errorRange}
                onChange={(e) => setErrorRange(e.target.value)}
                onFocus={() => handleInputFocus('errorRange')}
                onBlur={handleInputBlur}
              />
            </>
          )}
        </Box>
      </SidebarCard>

      {/* Plot Settings */}
      <SidebarCard title="Plot Settings" defaultExpanded={true}>
        {/* Plot Type */}
        <Box sx={{ mb: 2 }}>
          <Typography variant="caption" sx={{ color: 'rgba(255, 255, 255, 0.7)', fontSize: 10, fontWeight: 600, mb: 0.5, display: 'block' }}>
            PLOT TYPE
          </Typography>
          <FormControl>
            <RadioGroup
              row
              value={plotType}
              onChange={(e) => setPlotType(e.target.value as 'scatter' | 'line' | 'both')}
            >
              <FormControlLabel
                value="scatter"
                control={<Radio size="small" sx={{ py: 0.5, color: 'rgba(33, 150, 243, 0.5)', '&.Mui-checked': { color: anafisColors.spreadsheet } }} />}
                label={<Typography sx={{ fontSize: 11, color: 'rgba(255, 255, 255, 0.9)' }}>Scatter</Typography>}
              />
              <FormControlLabel
                value="line"
                control={<Radio size="small" sx={{ py: 0.5, color: 'rgba(33, 150, 243, 0.5)', '&.Mui-checked': { color: anafisColors.spreadsheet } }} />}
                label={<Typography sx={{ fontSize: 11, color: 'rgba(255, 255, 255, 0.9)' }}>Line</Typography>}
              />
              <FormControlLabel
                value="both"
                control={<Radio size="small" sx={{ py: 0.5, color: 'rgba(33, 150, 243, 0.5)', '&.Mui-checked': { color: anafisColors.spreadsheet } }} />}
                label={<Typography sx={{ fontSize: 11, color: 'rgba(255, 255, 255, 0.9)' }}>Both</Typography>}
              />
            </RadioGroup>
          </FormControl>
        </Box>

        {/* Update Plot Button */}
        <Button
          fullWidth
          variant="contained"
          startIcon={<PlotIcon />}
          onClick={handleUpdatePlot}
          disabled={isPlotting || !xRange || !yRange}
          sx={{
            mb: 2,
            bgcolor: anafisColors.spreadsheet,
            fontWeight: 600,
            fontSize: 12,
            py: 1,
            '&:hover': { bgcolor: '#1976d2' },
            '&:disabled': { bgcolor: '#424242' },
          }}
        >
          {isPlotting ? 'Plotting...' : 'Update Plot'}
        </Button>

        {/* Validation Info */}
        {hasPlot && !validationError && (
          <Alert severity="success" sx={{ mb: 2, py: 0.5, fontSize: 11 }}>
            ✓ {xData.length} points plotted
          </Alert>
        )}

        {validationError && (
          <Alert severity="error" onClose={() => setValidationError(null)} sx={{ mb: 2, py: 0.5, fontSize: 11 }}>
            {validationError}
          </Alert>
        )}
      </SidebarCard>

      {/* Plot Preview */}
      {hasPlot && (
        <SidebarCard title="Plot Preview" defaultExpanded={true}>
          <Box
            data-quick-plot
            sx={{
              bgcolor: 'rgba(0, 0, 0, 0.3)',
              border: '1px solid rgba(33, 150, 243, 0.3)',
              borderRadius: '6px',
              overflow: 'hidden',
            }}
          >
            <Box
              ref={chartRef}
              sx={{
                width: 388,
                height: 300,
              }}
            />
          </Box>
        </SidebarCard>
      )}

      {/* Export Actions */}
      {hasPlot && (
        <SidebarCard title="Export & Save" defaultExpanded={true}>
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
            <Button
              fullWidth
              variant="outlined"
              startIcon={<SaveIcon />}
              onClick={handleSaveToLibrary}
              sx={{
                color: anafisColors.spreadsheet,
                borderColor: 'rgba(33, 150, 243, 0.5)',
                '&:hover': { borderColor: anafisColors.spreadsheet, bgcolor: 'rgba(33, 150, 243, 0.1)' },
              }}
            >
              Save to Library
            </Button>
            <Button
              fullWidth
              variant="outlined"
              startIcon={<ExportIcon />}
              onClick={handleExportPNG}
              sx={{
                color: anafisColors.spreadsheet,
                borderColor: 'rgba(33, 150, 243, 0.5)',
                '&:hover': { borderColor: anafisColors.spreadsheet, bgcolor: 'rgba(33, 150, 243, 0.1)' },
              }}
            >
              Export Graph
            </Button>
          </Box>

          {/* Save to Library Dialog */}
          <Dialog open={saveDialogOpen} onClose={() => setSaveDialogOpen(false)} maxWidth="sm" fullWidth
            slotProps={{
              paper: {
                sx: {
                  bgcolor: '#1a1a1a',
                  backgroundImage: 'none',
                  color: '#ffffff',
                }
              }
            }}
          >
            <DialogTitle sx={{ color: anafisColors.spreadsheet, borderBottom: '1px solid rgba(33, 150, 243, 0.2)' }}>
              Save to Data Library
            </DialogTitle>
            <DialogContent sx={{ pt: 2 }}>
              <Typography variant="body2" gutterBottom sx={{ color: 'rgba(255, 255, 255, 0.8)' }}>
                Save X and Y sequences to the Data Library for later use.
              </Typography>

              <Box sx={{ mt: 2 }}>
                <Typography variant="subtitle2" gutterBottom sx={{ color: anafisColors.spreadsheet }}>X-Axis Sequence</Typography>
                <TextField
                  fullWidth
                  size="small"
                  label="Name"
                  value={xSequenceName}
                  onChange={(e) => setXSequenceName(e.target.value)}
                  sx={{ mb: 1 }}
                />
                <TextField
                  fullWidth
                  size="small"
                  label="Unit"
                  value={xUnit}
                  onChange={(e) => setXUnit(e.target.value)}
                />
              </Box>

              <Box sx={{ mt: 2 }}>
                <Typography variant="subtitle2" gutterBottom sx={{ color: anafisColors.spreadsheet }}>Y-Axis Sequence</Typography>
                <TextField
                  fullWidth
                  size="small"
                  label="Name"
                  value={ySequenceName}
                  onChange={(e) => setYSequenceName(e.target.value)}
                  sx={{ mb: 1 }}
                />
                <TextField
                  fullWidth
                  size="small"
                  label="Unit"
                  value={yUnit}
                  onChange={(e) => setYUnit(e.target.value)}
                />
              </Box>

              <Box sx={{ mt: 2 }}>
                <Typography variant="subtitle2" gutterBottom sx={{ color: anafisColors.spreadsheet }}>Tags</Typography>
                <Box sx={{ display: 'flex', gap: 0.5, flexWrap: 'wrap', mb: 1 }}>
                  {sequenceTags.map(tag => (
                    <Chip
                      key={tag}
                      label={tag}
                      size="small"
                      onDelete={() => handleRemoveTag(tag)}
                      sx={{
                        bgcolor: 'rgba(33, 150, 243, 0.2)',
                        color: anafisColors.spreadsheet,
                        '& .MuiChip-deleteIcon': { color: 'rgba(33, 150, 243, 0.7)' }
                      }}
                    />
                  ))}
                </Box>
                <Box sx={{ display: 'flex', gap: 1 }}>
                  <TextField
                    size="small"
                    placeholder="Add tag"
                    value={newTag}
                    onChange={(e) => setNewTag(e.target.value)}
                    onKeyPress={(e) => {
                      if (e.key === 'Enter') {
                        e.preventDefault();
                        handleAddTag();
                      }
                    }}
                    sx={{ flex: 1 }}
                  />
                  <Button size="small" onClick={handleAddTag} variant="outlined" sx={{ color: anafisColors.spreadsheet, borderColor: anafisColors.spreadsheet }}>
                    Add
                  </Button>
                </Box>
              </Box>
            </DialogContent>
            <DialogActions sx={{ borderTop: '1px solid rgba(33, 150, 243, 0.2)', p: 2 }}>
              <Button onClick={() => setSaveDialogOpen(false)} sx={{ color: 'rgba(255, 255, 255, 0.7)' }}>
                Cancel
              </Button>
              <Button onClick={handleConfirmSave} variant="contained" sx={{ bgcolor: anafisColors.spreadsheet, '&:hover': { bgcolor: '#1976d2' } }}>
                Save
              </Button>
            </DialogActions>
          </Dialog>

          {/* Export Dialog */}
          <Dialog open={exportDialogOpen} onClose={() => setExportDialogOpen(false)} maxWidth="xs" fullWidth
            slotProps={{
              paper: {
                sx: {
                  bgcolor: '#1a1a1a',
                  backgroundImage: 'none',
                  color: '#ffffff',
                }
              }
            }}
          >
            <DialogTitle sx={{ color: anafisColors.spreadsheet, borderBottom: '1px solid rgba(33, 150, 243, 0.2)' }}>
              Export Plot
            </DialogTitle>
            <DialogContent sx={{ pt: 3 }}>
              <Typography variant="body2" gutterBottom sx={{ color: 'rgba(255, 255, 255, 0.8)', mb: 2 }}>
                Choose export format and theme:
              </Typography>
              
              <Typography variant="caption" sx={{ color: 'rgba(255, 255, 255, 0.7)', fontSize: 10, fontWeight: 600, mb: 0.5, display: 'block' }}>
                FORMAT
              </Typography>
              <FormControl sx={{ mb: 2 }}>
                <RadioGroup
                  value={exportFormat}
                  onChange={(e) => setExportFormat(e.target.value as 'png' | 'svg')}
                >
                  <FormControlLabel
                    value="png"
                    control={<Radio size="small" sx={{ color: 'rgba(33, 150, 243, 0.5)', '&.Mui-checked': { color: anafisColors.spreadsheet } }} />}
                    label={<Typography sx={{ fontSize: 13, color: 'rgba(255, 255, 255, 0.9)' }}>PNG (Raster, 1200×800)</Typography>}
                  />
                  <FormControlLabel
                    value="svg"
                    control={<Radio size="small" sx={{ color: 'rgba(33, 150, 243, 0.5)', '&.Mui-checked': { color: anafisColors.spreadsheet } }} />}
                    label={<Typography sx={{ fontSize: 13, color: 'rgba(255, 255, 255, 0.9)' }}>SVG (Vector, scalable)</Typography>}
                  />
                </RadioGroup>
              </FormControl>
              
              <Typography variant="caption" sx={{ color: 'rgba(255, 255, 255, 0.7)', fontSize: 10, fontWeight: 600, mb: 0.5, display: 'block' }}>
                THEME
              </Typography>
              <FormControl>
                <RadioGroup
                  value={exportTheme}
                  onChange={(e) => setExportTheme(e.target.value as 'dark' | 'light')}
                >
                  <FormControlLabel
                    value="dark"
                    control={<Radio size="small" sx={{ color: 'rgba(33, 150, 243, 0.5)', '&.Mui-checked': { color: anafisColors.spreadsheet } }} />}
                    label={<Typography sx={{ fontSize: 13, color: 'rgba(255, 255, 255, 0.9)' }}>Dark background</Typography>}
                  />
                  <FormControlLabel
                    value="light"
                    control={<Radio size="small" sx={{ color: 'rgba(33, 150, 243, 0.5)', '&.Mui-checked': { color: anafisColors.spreadsheet } }} />}
                    label={<Typography sx={{ fontSize: 13, color: 'rgba(255, 255, 255, 0.9)' }}>Light background</Typography>}
                  />
                </RadioGroup>
              </FormControl>
            </DialogContent>
            <DialogActions sx={{ borderTop: '1px solid rgba(33, 150, 243, 0.2)', p: 2 }}>
              <Button 
                onClick={() => setExportDialogOpen(false)} 
                disabled={isExporting}
                sx={{ color: 'rgba(255, 255, 255, 0.7)' }}
              >
                Cancel
              </Button>
              <Button 
                onClick={handleConfirmExport} 
                disabled={isExporting}
                variant="contained" 
                sx={{ bgcolor: anafisColors.spreadsheet, '&:hover': { bgcolor: '#1976d2' } }}
                startIcon={isExporting ? <CircularProgress size={20} sx={{ color: 'white' }} /> : null}
              >
                {isExporting ? 'Exporting...' : 'Export'}
              </Button>
            </DialogActions>
          </Dialog>
        </SidebarCard>
      )}
      </Box>
    </Box>
  );
};
export default QuickPlotSidebar;
