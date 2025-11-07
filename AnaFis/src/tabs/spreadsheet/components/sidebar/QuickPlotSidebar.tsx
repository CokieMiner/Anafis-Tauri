import React from 'react';
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
import { SpreadsheetRef } from '@/tabs/spreadsheet/types/SpreadsheetInterface';
import { sidebarStyles } from '@/tabs/spreadsheet/components/sidebar/utils/sidebarStyles';
import SidebarCard from '@/tabs/spreadsheet/components/sidebar/SidebarCard';
import { useSpreadsheetSelection } from '@/tabs/spreadsheet/managers/useSpreadsheetSelection';
import { anafisColors } from '@/tabs/spreadsheet/components/sidebar/themes';
import { spreadsheetEventBus } from '@/tabs/spreadsheet/managers/SpreadsheetEventBus';
import { useQuickPlot } from '@/tabs/spreadsheet/components/sidebar/logic/useQuickPlot';

// Icon aliases for clarity
const PlotIcon = ShowChartIcon;
const ExportIcon = ImageIcon;

interface QuickPlotSidebarProps {
  open: boolean;
  onClose: () => void;
  spreadsheetRef?: React.RefObject<SpreadsheetRef | null>;
  onSelectionChange?: (selection: string) => void;
}

type FocusedInputType = 'xRange' | 'yRange' | 'errorRange' | null;

const QuickPlotSidebar = React.memo<QuickPlotSidebarProps>(({
  open,
  onClose,
  spreadsheetRef,
  onSelectionChange,
}) => {
  // Use the QuickPlot hook for all plotting logic
  const {
    // Configuration state
    xRange,
    yRange,
    errorRange,
    xLabel,
    yLabel,
    plotType,
    showErrorBars,
    xData,
    isGenerating,
    error,
    hasPlot,
    chartRef,

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
  } = useQuickPlot({
    spreadsheetRef: spreadsheetRef ?? { current: null },
    onSelectionChange: onSelectionChange ?? (() => {}),
  });

  // Use the spreadsheet selection hook
  const { focusedInput, handleInputFocus, handleInputBlur } = useSpreadsheetSelection<FocusedInputType>({
    onSelectionChange: onSelectionChange ?? (() => { }),
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

  // Subscribe to spreadsheet selection events via event bus
  React.useEffect(() => {
    if (!open) { return; }

    const unsubscribe = spreadsheetEventBus.on('selection-change', (cellRef) => {
      // Call the window handler that the hook is listening to
      const handler = window.__quickPlotSelectionHandler;
      if (handler) {
        handler(cellRef);
      }
      // NOTE: Don't call onSelectionChange here - it would create an infinite loop
      // since onSelectionChange emits to the event bus, which triggers this handler again
    });

    return unsubscribe;
  }, [open]);

  // Handle save to library
  const handleSaveToLibraryClick = () => {
    // Pre-fill with labels if available
    if (!xSequenceName && xLabel) { setXSequenceName(xLabel); }
    if (!ySequenceName && yLabel) { setYSequenceName(yLabel); }
    setSaveDialogOpen(true);
  };

  if (!open) { return null; }

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
      <Box sx={sidebarStyles.contentWrapper}>
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
              sx={{
                mb: 1,
                '& .MuiOutlinedInput-root': {
                  bgcolor: focusedInput === 'xRange' ? 'rgba(33, 150, 243, 0.1)' : 'rgba(33, 150, 243, 0.05)',
                  borderRadius: '6px',
                  '& fieldset': { borderColor: focusedInput === 'xRange' ? anafisColors.spreadsheet : 'rgba(33, 150, 243, 0.2)' },
                  '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                  '&.Mui-focused fieldset': { borderColor: anafisColors.spreadsheet },
                  '& input': { color: 'white', fontFamily: 'monospace', fontSize: 13 }
                }
              }}
            />
            <TextField
              fullWidth
              size="small"
              placeholder="X-axis label"
              value={xLabel}
              onChange={(e) => setXLabel(e.target.value)}
              sx={{
                '& .MuiOutlinedInput-root': {
                  bgcolor: 'rgba(33, 150, 243, 0.05)',
                  borderRadius: '6px',
                  '& fieldset': { borderColor: 'rgba(33, 150, 243, 0.2)' },
                  '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                  '&.Mui-focused fieldset': { borderColor: anafisColors.spreadsheet },
                  '& input': { color: 'white', fontSize: 13 }
                }
              }}
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
              sx={{
                mb: 1,
                '& .MuiOutlinedInput-root': {
                  bgcolor: focusedInput === 'yRange' ? 'rgba(33, 150, 243, 0.1)' : 'rgba(33, 150, 243, 0.05)',
                  borderRadius: '6px',
                  '& fieldset': { borderColor: focusedInput === 'yRange' ? anafisColors.spreadsheet : 'rgba(33, 150, 243, 0.2)' },
                  '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                  '&.Mui-focused fieldset': { borderColor: anafisColors.spreadsheet },
                  '& input': { color: 'white', fontFamily: 'monospace', fontSize: 13 }
                }
              }}
            />
            <TextField
              fullWidth
              size="small"
              placeholder="Y-axis label"
              value={yLabel}
              onChange={(e) => setYLabel(e.target.value)}
              sx={{
                '& .MuiOutlinedInput-root': {
                  bgcolor: 'rgba(33, 150, 243, 0.05)',
                  borderRadius: '6px',
                  '& fieldset': { borderColor: 'rgba(33, 150, 243, 0.2)' },
                  '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                  '&.Mui-focused fieldset': { borderColor: anafisColors.spreadsheet },
                  '& input': { color: 'white', fontSize: 13 }
                }
              }}
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
                  sx={{
                    '& .MuiOutlinedInput-root': {
                      bgcolor: focusedInput === 'errorRange' ? 'rgba(33, 150, 243, 0.1)' : 'rgba(33, 150, 243, 0.05)',
                      borderRadius: '6px',
                      '& fieldset': { borderColor: focusedInput === 'errorRange' ? anafisColors.spreadsheet : 'rgba(33, 150, 243, 0.2)' },
                      '&:hover fieldset': { borderColor: 'rgba(33, 150, 243, 0.4)' },
                      '&.Mui-focused fieldset': { borderColor: anafisColors.spreadsheet },
                      '& input': { color: 'white', fontFamily: 'monospace', fontSize: 13 }
                    }
                  }}
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
            onClick={() => void generatePlot()}
            disabled={isGenerating || !xRange || !yRange}
            sx={{
              ...sidebarStyles.button.primary,
              mb: 2,
              fontSize: 12,
              py: 1
            }}
          >
            {isGenerating ? 'Plotting...' : 'Update Plot'}
          </Button>

          {/* Validation Info */}
          {hasPlot && !error && (
            <Alert severity="success" sx={{ mb: 2, py: 0.5, fontSize: 11 }}>
              ✓ {xData.length} points plotted
            </Alert>
          )}

          {error && (
            <Alert severity="error" onClose={clearError} sx={{ mb: 2, py: 0.5, fontSize: 11 }}>
              {error}
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
                onClick={handleSaveToLibraryClick}
                sx={sidebarStyles.button.secondary}
              >
                Save to Library
              </Button>
              <Button
                fullWidth
                variant="outlined"
                startIcon={<ExportIcon />}
                onClick={() => setExportDialogOpen(true)}
                sx={sidebarStyles.button.secondary}
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
                        onDelete={() => removeTag(tag)}
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
                          addTag();
                        }
                      }}
                      sx={{ flex: 1 }}
                    />
                    <Button size="small" onClick={addTag} variant="outlined" sx={sidebarStyles.button.secondary}>
                      Add
                    </Button>
                  </Box>
                </Box>
              </DialogContent>
              <DialogActions sx={{ borderTop: '1px solid rgba(33, 150, 243, 0.2)', p: 2 }}>
                <Button onClick={() => setSaveDialogOpen(false)} sx={{ ...sidebarStyles.button.secondary, backgroundColor: 'transparent' }}>
                  Cancel
                </Button>
                <Button onClick={() => { void handleSaveToLibrary(); }} variant="contained" sx={sidebarStyles.button.primary}>
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
                  sx={{ ...sidebarStyles.button.secondary, backgroundColor: 'transparent' }}
                >
                  Cancel
                </Button>
                <Button
                  onClick={() => { void handleExport(); }}
                  disabled={isExporting}
                  variant="contained"
                  sx={sidebarStyles.button.primary}
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
});

QuickPlotSidebar.displayName = 'QuickPlotSidebar';

export default QuickPlotSidebar;