import React, { useState } from 'react';
import {
  Paper,
  Box,
  Typography,
  IconButton,
  TextField,
  Button,
  Alert,
  Radio,
  RadioGroup,
  FormControlLabel,
  FormControl,
  Checkbox,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Chip,
} from '@mui/material';
import {
  Close as CloseIcon,
  BarChart as PlotIcon,
  Save as SaveIcon,
  Image as ExportIcon,
  Fullscreen as FullscreenIcon,
} from '@mui/icons-material';
import Plot from 'react-plotly.js';
import { invoke } from '@tauri-apps/api/core';
import { UniverSpreadsheetRef } from './UniverSpreadsheet';
import { useSpreadsheetSelection } from '../../hooks/useSpreadsheetSelection';

interface QuickPlotSidebarProps {
  open: boolean;
  onClose: () => void;
  univerRef?: React.RefObject<UniverSpreadsheetRef | null>;
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
  // State for plot data
  const [xData, setXData] = useState<number[]>([]);
  const [yData, setYData] = useState<number[]>([]);
  const [errorData, setErrorData] = useState<number[] | undefined>(undefined);
  const [validationError, setValidationError] = useState<string | null>(null);
  const [isPlotting, setIsPlotting] = useState(false);
  const [hasPlot, setHasPlot] = useState(false);
  
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
  
  // Full screen plot state
  const [fullScreenOpen, setFullScreenOpen] = useState(false);
  
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
      const xFlat = xValues.flat().map((v: any) => {
        const num = typeof v === 'number' ? v : parseFloat(String(v));
        return num;
      });

      // Get Y data
      const yValues = await univerRef.current.getRange(yRange);
      const yFlat = yValues.flat().map((v: any) => {
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
      if (xFlat.some((v: any) => !isFinite(v))) {
        setValidationError('X data contains invalid values (NaN or Infinity)');
        return false;
      }

      if (yFlat.some((v: any) => !isFinite(v))) {
        setValidationError('Y data contains invalid values (NaN or Infinity)');
        return false;
      }

      // Get error bars if enabled
      let errorFlat: number[] | undefined = undefined;
      if (showErrorBars && errorRange) {
        const errorValues = await univerRef.current.getRange(errorRange);
        errorFlat = errorValues.flat().map((v: any) => {
          const num = typeof v === 'number' ? v : parseFloat(String(v));
          return Math.abs(num);
        });

        if (errorFlat && errorFlat.length !== yFlat.length) {
          setValidationError(`Error range length (${errorFlat.length}) doesn't match Y data (${yFlat.length})`);
          return false;
        }

        if (errorFlat && errorFlat.some((v: any) => !isFinite(v))) {
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
    await extractAndValidateData();
  };

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
    if (!hasPlot || plotData.length === 0) return;
    
    try {
      // @ts-ignore - plotly.js-dist-min doesn't have types
      const Plotly = await import('plotly.js-dist-min');
      
      // Determine colors based on theme
      const bgColor = exportTheme === 'light' ? '#ffffff' : '#0a0a0a';
      const textColor = exportTheme === 'light' ? '#000000' : '#ffffff';
      const gridColor = exportTheme === 'light' ? 'rgba(0,0,0,0.1)' : 'rgba(255,255,255,0.1)';
      const plotBgColor = exportTheme === 'light' ? '#f5f5f5' : 'rgba(0,0,0,0.3)';
      
      // Create a temporary div for rendering with custom theme
      const tempDiv = document.createElement('div');
      tempDiv.style.width = '1200px';
      tempDiv.style.height = '800px';
      tempDiv.style.position = 'absolute';
      tempDiv.style.left = '-9999px';
      document.body.appendChild(tempDiv);
      
      // Update plot data colors for light theme if needed
      const exportPlotData = plotData.map((trace: any) => ({
        ...trace,
        marker: exportTheme === 'light' && trace.marker ? {
          ...trace.marker,
          color: trace.mode === 'markers' ? '#1976d2' : trace.marker.color
        } : trace.marker,
        line: exportTheme === 'light' && trace.line ? {
          ...trace.line,
          color: '#1976d2'
        } : trace.line,
      }));
      
      await (Plotly as any).newPlot(tempDiv, exportPlotData, {
        width: 1200,
        height: 800,
        paper_bgcolor: bgColor,
        plot_bgcolor: plotBgColor,
        font: { color: textColor, size: 14 },
        xaxis: { 
          title: { text: xLabel || 'X' }, 
          gridcolor: gridColor,
          color: textColor,
        },
        yaxis: { 
          title: { text: yLabel || 'Y' }, 
          gridcolor: gridColor,
          color: textColor,
        },
        margin: { l: 80, r: 40, t: 40, b: 80 },
        showlegend: plotType === 'both',
        legend: { 
          bgcolor: exportTheme === 'light' ? 'rgba(255,255,255,0.8)' : 'rgba(0,0,0,0.5)',
          font: { color: textColor }
        },
      }, {
        staticPlot: true,
      });
      
      // Download the image
      await (Plotly as any).downloadImage(tempDiv, {
        format: 'png',
        width: 1200,
        height: 800,
        filename: `quick_plot_${Date.now()}`,
      });
      
      // Clean up
      document.body.removeChild(tempDiv);
      setExportDialogOpen(false);
    } catch (error) {
      console.error('Export failed:', error);
      setValidationError('PNG export failed. Try using the camera icon in the plot toolbar instead.');
      setExportDialogOpen(false);
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

  // Prepare Plotly data
  const plotData: any[] = [];
  
  if (hasPlot && xData.length > 0 && yData.length > 0) {
    if (plotType === 'scatter' || plotType === 'both') {
      plotData.push({
        x: xData,
        y: yData,
        mode: 'markers',
        type: 'scatter',
        name: yLabel || 'Data',
        marker: { size: 8, color: '#2196f3' },
        error_y: showErrorBars && errorData ? {
          type: 'data',
          array: errorData,
          visible: true,
          color: '#f44336',
        } : undefined,
      });
    }
    
    if (plotType === 'line' || plotType === 'both') {
      plotData.push({
        x: xData,
        y: yData,
        mode: 'lines',
        type: 'scatter',
        name: plotType === 'both' ? 'Trend' : yLabel || 'Data',
        line: { width: 2, color: plotType === 'both' ? '#4caf50' : '#2196f3' },
      });
    }
  }

  return (
    <Paper
      data-quick-plot-sidebar
      elevation={3}
      sx={{
        width: 420,
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        bgcolor: 'rgba(10, 25, 45, 0.98)',
        border: '1px solid rgba(33, 150, 243, 0.2)',
        borderLeft: '2px solid rgba(33, 150, 243, 0.5)',
        borderRadius: 0,
        overflow: 'hidden',
      }}
    >
      {/* Header */}
      <Box
        sx={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          p: 2,
          bgcolor: 'rgba(33, 150, 243, 0.08)',
          borderBottom: '1px solid rgba(33, 150, 243, 0.2)',
        }}
      >
        <Typography variant="h6" sx={{ fontWeight: 600, color: '#2196f3' }}>
          Quick Plot
        </Typography>
        <IconButton
          onClick={onClose}
          size="small"
          sx={{
            color: 'rgba(255, 255, 255, 0.7)',
            borderRadius: '6px',
            '&:hover': {
              bgcolor: 'rgba(33, 150, 243, 0.2)',
              color: 'rgba(255, 255, 255, 0.9)',
            },
          }}
        >
          <CloseIcon />
        </IconButton>
      </Box>

      {/* Tip */}
      <Box
        sx={{
          p: 1.5,
          bgcolor: 'rgba(33, 150, 243, 0.08)',
          borderBottom: '1px solid rgba(33, 150, 243, 0.15)',
        }}
      >
        <Typography variant="caption" sx={{ color: 'rgba(255, 255, 255, 0.8)', fontSize: 11, lineHeight: 1.4 }}>
          📊 Quick 2D plotting for data exploration
        </Typography>
      </Box>

      {/* Main Content */}
      <Box sx={{ flex: 1, overflow: 'auto', p: 2 }}>
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
                sx={{ color: 'rgba(33, 150, 243, 0.5)', '&.Mui-checked': { color: '#2196f3' } }}
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
                control={<Radio size="small" sx={{ py: 0.5, color: 'rgba(33, 150, 243, 0.5)', '&.Mui-checked': { color: '#2196f3' } }} />}
                label={<Typography sx={{ fontSize: 11, color: 'rgba(255, 255, 255, 0.9)' }}>Scatter</Typography>}
              />
              <FormControlLabel
                value="line"
                control={<Radio size="small" sx={{ py: 0.5, color: 'rgba(33, 150, 243, 0.5)', '&.Mui-checked': { color: '#2196f3' } }} />}
                label={<Typography sx={{ fontSize: 11, color: 'rgba(255, 255, 255, 0.9)' }}>Line</Typography>}
              />
              <FormControlLabel
                value="both"
                control={<Radio size="small" sx={{ py: 0.5, color: 'rgba(33, 150, 243, 0.5)', '&.Mui-checked': { color: '#2196f3' } }} />}
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
            bgcolor: '#2196f3',
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

        {/* Plot Preview */}
        {hasPlot && plotData.length > 0 && (
          <Box
            data-quick-plot
            sx={{
              mb: 2,
              bgcolor: 'rgba(0, 0, 0, 0.3)',
              border: '1px solid rgba(33, 150, 243, 0.3)',
              borderRadius: '6px',
              overflow: 'hidden',
            }}
          >
            <Plot
              data={plotData}
              layout={{
                width: 388,
                height: 300,
                paper_bgcolor: 'rgba(0,0,0,0)',
                plot_bgcolor: 'rgba(0,0,0,0.3)',
                font: { color: '#ffffff' },
                xaxis: { title: { text: xLabel || 'X' }, gridcolor: 'rgba(255,255,255,0.1)' },
                yaxis: { title: { text: yLabel || 'Y' }, gridcolor: 'rgba(255,255,255,0.1)' },
                margin: { l: 50, r: 20, t: 20, b: 50 },
                showlegend: plotType === 'both',
                legend: { bgcolor: 'rgba(0,0,0,0.5)' },
              }}
              config={{
                displayModeBar: true,
                displaylogo: false,
                modeBarButtonsToRemove: ['lasso2d', 'select2d'],
              }}
            />
          </Box>
        )}

        {/* Actions */}
        {hasPlot && (
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
            <Button
              fullWidth
              variant="outlined"
              startIcon={<FullscreenIcon />}
              onClick={() => setFullScreenOpen(true)}
              sx={{
                color: '#2196f3',
                borderColor: 'rgba(33, 150, 243, 0.5)',
                '&:hover': { borderColor: '#2196f3', bgcolor: 'rgba(33, 150, 243, 0.1)' },
              }}
            >
              View Fullscreen
            </Button>
            <Button
              fullWidth
              variant="outlined"
              startIcon={<SaveIcon />}
              onClick={handleSaveToLibrary}
              sx={{
                color: '#2196f3',
                borderColor: 'rgba(33, 150, 243, 0.5)',
                '&:hover': { borderColor: '#2196f3', bgcolor: 'rgba(33, 150, 243, 0.1)' },
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
                color: '#2196f3',
                borderColor: 'rgba(33, 150, 243, 0.5)',
                '&:hover': { borderColor: '#2196f3', bgcolor: 'rgba(33, 150, 243, 0.1)' },
              }}
            >
              Export PNG
            </Button>
          </Box>
        )}
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
        <DialogTitle sx={{ color: '#2196f3', borderBottom: '1px solid rgba(33, 150, 243, 0.2)' }}>
          Save to Data Library
        </DialogTitle>
        <DialogContent sx={{ pt: 2 }}>
          <Typography variant="body2" gutterBottom sx={{ color: 'rgba(255, 255, 255, 0.8)' }}>
            Save X and Y sequences to the Data Library for later use.
          </Typography>

          <Box sx={{ mt: 2 }}>
            <Typography variant="subtitle2" gutterBottom sx={{ color: '#2196f3' }}>X-Axis Sequence</Typography>
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
            <Typography variant="subtitle2" gutterBottom sx={{ color: '#2196f3' }}>Y-Axis Sequence</Typography>
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
            <Typography variant="subtitle2" gutterBottom sx={{ color: '#2196f3' }}>Tags</Typography>
            <Box sx={{ display: 'flex', gap: 0.5, flexWrap: 'wrap', mb: 1 }}>
              {sequenceTags.map(tag => (
                <Chip
                  key={tag}
                  label={tag}
                  size="small"
                  onDelete={() => handleRemoveTag(tag)}
                  sx={{
                    bgcolor: 'rgba(33, 150, 243, 0.2)',
                    color: '#2196f3',
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
              <Button size="small" onClick={handleAddTag} variant="outlined" sx={{ color: '#2196f3', borderColor: '#2196f3' }}>
                Add
              </Button>
            </Box>
          </Box>
        </DialogContent>
        <DialogActions sx={{ borderTop: '1px solid rgba(33, 150, 243, 0.2)', p: 2 }}>
          <Button onClick={() => setSaveDialogOpen(false)} sx={{ color: 'rgba(255, 255, 255, 0.7)' }}>
            Cancel
          </Button>
          <Button onClick={handleConfirmSave} variant="contained" sx={{ bgcolor: '#2196f3', '&:hover': { bgcolor: '#1976d2' } }}>
            Save
          </Button>
        </DialogActions>
      </Dialog>

      {/* Export PNG Dialog */}
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
        <DialogTitle sx={{ color: '#2196f3', borderBottom: '1px solid rgba(33, 150, 243, 0.2)' }}>
          Export as PNG
        </DialogTitle>
        <DialogContent sx={{ pt: 3 }}>
          <Typography variant="body2" gutterBottom sx={{ color: 'rgba(255, 255, 255, 0.8)', mb: 2 }}>
            Choose the background theme for the exported image:
          </Typography>
          <FormControl>
            <RadioGroup
              value={exportTheme}
              onChange={(e) => setExportTheme(e.target.value as 'dark' | 'light')}
            >
              <FormControlLabel
                value="dark"
                control={<Radio sx={{ color: 'rgba(33, 150, 243, 0.5)', '&.Mui-checked': { color: '#2196f3' } }} />}
                label={<Typography sx={{ color: 'rgba(255, 255, 255, 0.9)' }}>Dark background (current)</Typography>}
              />
              <FormControlLabel
                value="light"
                control={<Radio sx={{ color: 'rgba(33, 150, 243, 0.5)', '&.Mui-checked': { color: '#2196f3' } }} />}
                label={<Typography sx={{ color: 'rgba(255, 255, 255, 0.9)' }}>Light background (white)</Typography>}
              />
            </RadioGroup>
          </FormControl>
        </DialogContent>
        <DialogActions sx={{ borderTop: '1px solid rgba(33, 150, 243, 0.2)', p: 2 }}>
          <Button onClick={() => setExportDialogOpen(false)} sx={{ color: 'rgba(255, 255, 255, 0.7)' }}>
            Cancel
          </Button>
          <Button onClick={handleConfirmExport} variant="contained" sx={{ bgcolor: '#2196f3', '&:hover': { bgcolor: '#1976d2' } }}>
            Export
          </Button>
        </DialogActions>
      </Dialog>

      {/* Fullscreen Plot Dialog */}
      <Dialog 
        open={fullScreenOpen} 
        onClose={() => setFullScreenOpen(false)} 
        maxWidth={false}
        fullWidth
        slotProps={{
          paper: {
            sx: {
              bgcolor: '#0a0a0a',
              backgroundImage: 'none',
              height: '90vh',
              maxWidth: '90vw',
              m: 2,
            }
          }
        }}
      >
        <DialogTitle sx={{ 
          color: '#2196f3', 
          borderBottom: '1px solid rgba(33, 150, 243, 0.2)',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center'
        }}>
          <span>Quick Plot - Fullscreen View</span>
          <IconButton onClick={() => setFullScreenOpen(false)} sx={{ color: 'rgba(255, 255, 255, 0.7)' }}>
            <CloseIcon />
          </IconButton>
        </DialogTitle>
        <DialogContent sx={{ display: 'flex', alignItems: 'center', justifyContent: 'center', p: 3 }}>
          {hasPlot && plotData.length > 0 && (
            <Plot
              data={plotData}
              layout={{
                autosize: true,
                paper_bgcolor: 'rgba(0,0,0,0)',
                plot_bgcolor: 'rgba(0,0,0,0.3)',
                font: { color: '#ffffff', size: 14 },
                xaxis: { title: { text: xLabel || 'X' }, gridcolor: 'rgba(255,255,255,0.1)' },
                yaxis: { title: { text: yLabel || 'Y' }, gridcolor: 'rgba(255,255,255,0.1)' },
                margin: { l: 80, r: 40, t: 40, b: 80 },
                showlegend: plotType === 'both',
                legend: { bgcolor: 'rgba(0,0,0,0.5)' },
              }}
              config={{
                displayModeBar: true,
                displaylogo: false,
                modeBarButtonsToRemove: ['lasso2d', 'select2d'],
                responsive: true,
              }}
              style={{ width: '100%', height: '100%' }}
              useResizeHandler={true}
            />
          )}
        </DialogContent>
      </Dialog>
    </Paper>
  );
};

export default QuickPlotSidebar;
