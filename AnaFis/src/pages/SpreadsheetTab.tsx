import React, { useState, useCallback, useRef, useMemo } from 'react';
import {
  Box,
  Button,
  Toolbar,
  Paper,
  SxProps,
  Theme
} from '@mui/material';
import {
  Transform as UnitConverterIcon,
  AutoFixHigh as UncertaintyIcon,
  BarChart as QuickPlotIcon,
  FileDownload as ExportIcon
} from '@mui/icons-material';

import { UniverAdapter } from '../components/spreadsheet/univer';
import { SpreadsheetRef, WorkbookData, CellValue } from '../components/spreadsheet/SpreadsheetInterface';
import { spreadsheetEventBus } from '../components/spreadsheet/SpreadsheetEventBus';

import UncertaintySidebar from '../components/spreadsheet/sidebar/UncertaintySidebar';
import UnitConversionSidebar from '../components/spreadsheet/sidebar/UnitConversionSidebar';
import QuickPlotSidebar from '../components/spreadsheet/sidebar/QuickPlotSidebar';
import ExportSidebar from '../components/spreadsheet/sidebar/ExportSidebar';
import { ExportFormat, ExportRangeMode, JsonFormat } from '../types/export';
import { anafisColors } from '../themes';

interface Variable {
  name: string;
  valueRange: string;
  uncertaintyRange: string;
  confidence: number;
}

// Consolidated sidebar state interface for better organization
interface SidebarState {

  // Uncertainty sidebar
  uncertaintyVariables: Variable[];
  uncertaintyFormula: string;
  uncertaintyOutputValueRange: string;
  uncertaintyOutputUncertaintyRange: string;
  uncertaintyOutputConfidence: number;

  // Unit conversion sidebar
  unitConversionCategory: string;
  unitConversionFromUnit: string;
  unitConversionToUnit: string;
  unitConversionValue: string;

  // Quick Plot sidebar
  quickPlotXRange: string;
  quickPlotYRange: string;
  quickPlotErrorRange: string;
  quickPlotXLabel: string;
  quickPlotYLabel: string;
  quickPlotType: 'scatter' | 'line' | 'both';
  quickPlotShowErrorBars: boolean;

  // Export sidebar
  exportFormat: ExportFormat;
  exportRangeMode: ExportRangeMode;
  exportCustomRange: string;
  exportJsonFormat: JsonFormat;
  exportPrettyPrint: boolean;
  exportCustomDelimiter: string;
}

interface SpreadsheetTabProps {
  tabId: string;
}

const SpreadsheetTab: React.FC<SpreadsheetTabProps> = ({ tabId }) => {
  // Check if we're in a detached window by looking at URL params
  const isDetachedWindow = new URLSearchParams(window.location.search).has('tabId');

  // Sidebar state management - now using simple local state since tabs stay mounted
  type SidebarType = 'uncertainty' | 'unitConvert' | 'quickPlot' | 'export' | null;
  const [activeSidebar, setActiveSidebar] = useState<SidebarType>(null);

  // Memoized button styles for performance - create stable style objects for active/inactive states
  const toolbarButtonStyles = useMemo(() => ({
    active: {
      mr: 1,
      color: anafisColors.spreadsheet,
      borderColor: anafisColors.spreadsheet,
      backgroundColor: 'rgba(33, 150, 243, 0.2)',
      outline: 'none',
      '&:hover': {
        borderColor: anafisColors.spreadsheet,
        backgroundColor: 'rgba(33, 150, 243, 0.3)'
      },
      '&:focus': {
        borderColor: anafisColors.spreadsheet,
        outline: 'none',
      },
      '&:focus-visible': {
        borderColor: anafisColors.spreadsheet,
        outline: 'none',
        boxShadow: '0 0 0 2px rgba(33, 150, 243, 0.5)',
      },
      '&:active': {
        borderColor: anafisColors.spreadsheet,
        outline: 'none',
      }
    } as SxProps<Theme>,
    inactive: {
      mr: 1,
      color: 'white',
      borderColor: '#64b5f6',
      backgroundColor: 'transparent',
      outline: 'none',
      '&:hover': {
        borderColor: anafisColors.spreadsheet,
        backgroundColor: 'rgba(33, 150, 243, 0.1)'
      },
      '&:focus': {
        borderColor: anafisColors.spreadsheet,
        outline: 'none',
      },
      '&:focus-visible': {
        borderColor: anafisColors.spreadsheet,
        outline: 'none',
        boxShadow: '0 0 0 2px rgba(33, 150, 243, 0.5)',
      },
      '&:active': {
        borderColor: anafisColors.spreadsheet,
        outline: 'none',
      }
    } as SxProps<Theme>
  }), []);

  // Consolidated sidebar state - better performance and organization
  const [sidebarState, setSidebarState] = useState<SidebarState>({
    // Uncertainty sidebar defaults
    uncertaintyVariables: [{ name: 'a', valueRange: 'A1:A10', uncertaintyRange: 'B1:B10', confidence: 95 }],
    uncertaintyFormula: '',
    uncertaintyOutputValueRange: 'C1:C10',
    uncertaintyOutputUncertaintyRange: 'D1:D10',
    uncertaintyOutputConfidence: 95,

    // Unit conversion sidebar defaults
    unitConversionCategory: '',
    unitConversionFromUnit: '',
    unitConversionToUnit: '',
    unitConversionValue: '1',

    // Quick Plot sidebar defaults
    quickPlotXRange: '',
    quickPlotYRange: '',
    quickPlotErrorRange: '',
    quickPlotXLabel: '',
    quickPlotYLabel: '',
    quickPlotType: 'scatter',
    quickPlotShowErrorBars: false,

    // Export sidebar defaults
    exportFormat: 'csv',
    exportRangeMode: 'sheet',
    exportCustomRange: '',
    exportJsonFormat: 'records',
    exportPrettyPrint: true,
    exportCustomDelimiter: '|'
  });

  // Spreadsheet state - now persistent per tab
  const spreadsheetRef = useRef<SpreadsheetRef>(null);

  // NOTE: Tab synchronization removed - tabs are now local-only

  // Sidebar state is now purely local - no backend persistence

  // Helper function to update sidebar state (local only)
  const updateSidebarState = useCallback(<K extends keyof SidebarState>(
    key: K,
    value: SidebarState[K]
  ) => {
    setSidebarState(prev => ({ ...prev, [key]: value }));
  }, []);

  // Create initial workbook data - only load once to avoid circular updates
  const [spreadsheetData] = useState<WorkbookData>(() => {
    const sheetId = 'sheet-01';
    const workbookId = `workbook-${tabId}`;

    return {
      id: workbookId,
      name: `AnaFis Spreadsheet - ${tabId}`,
      appVersion: '1.0.0',
      locale: 'EN_US',
      styles: {},
      sheets: {
        [sheetId]: {
          id: sheetId,
          name: 'Sheet1',
          cellData: {},
          rowCount: 1000,
          columnCount: 26,
        }
      },
      sheetOrder: [sheetId],
    };
  });

  // NOTE: Backend data persistence removed - spreadsheet data is now local-only

  const handleCellChange = useCallback((cellRef: string, value: CellValue) => {
    // Cell change handler - Univer manages all data internally
    // NOTE: Backend persistence removed - cell data is now local-only
    console.log('Cell changed:', cellRef, value);
  }, []);

  const handleFormulaIntercept = useCallback((cellRef: string, formula: string) => {
    // Formula interception is no longer needed - Univer handles all formulas
    // Custom functions are registered directly with Univer's formula engine
    // This handler is kept for potential future use (e.g., formula validation)
    console.log('Formula entered:', cellRef, formula);
  }, []);

  const handleSelectionChange = useCallback((cellRef: string) => {
    // Emit selection change event to all interested subscribers (sidebars)
    spreadsheetEventBus.emit('selection-change', cellRef);
  }, []);

  // No initialization needed - tabs stay mounted so state persists naturally

  // Handlers
  const handleOpenUnitConverter = useCallback(() => {
    // Toggle sidebar - if already open, close it; otherwise open it
    setActiveSidebar(prev => prev === 'unitConvert' ? null : 'unitConvert');
  }, []);

  const handleOpenUncertaintyPropagation = useCallback(() => {
    // Toggle sidebar - if already open, close it; otherwise open it
    setActiveSidebar(prev => prev === 'uncertainty' ? null : 'uncertainty');
  }, []);

  const handleOpenQuickPlot = useCallback(() => {
    // Toggle sidebar - if already open, close it; otherwise open it
    setActiveSidebar(prev => prev === 'quickPlot' ? null : 'quickPlot');
  }, []);

  const handleOpenExport = useCallback(() => {
    // Toggle sidebar - if already open, close it; otherwise open it
    setActiveSidebar(prev => prev === 'export' ? null : 'export');
  }, []);

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* NOTE: Conflict resolution and error UI removed - no longer applicable for local-only tabs */}

      {/* Main Toolbar - show in both main window and detached windows */}
      <Paper sx={{ mb: 1, bgcolor: '#0a0a0a' }}>
        <Toolbar variant="dense" sx={{ minHeight: 48 }}>
          <Button
            variant="outlined"
            size="small"
            startIcon={<QuickPlotIcon />}
            onClick={handleOpenQuickPlot}
            sx={activeSidebar === 'quickPlot' ? toolbarButtonStyles.active : toolbarButtonStyles.inactive}
          >
            Quick Plot
          </Button>

          <Button
            variant="outlined"
            size="small"
            startIcon={<UncertaintyIcon />}
            onClick={handleOpenUncertaintyPropagation}
            sx={activeSidebar === 'uncertainty' ? toolbarButtonStyles.active : toolbarButtonStyles.inactive}
          >
            Uncertainty Propagation
          </Button>

          <Button
            variant="outlined"
            size="small"
            startIcon={<UnitConverterIcon />}
            onClick={handleOpenUnitConverter}
            sx={activeSidebar === 'unitConvert' ? toolbarButtonStyles.active : toolbarButtonStyles.inactive}
          >
            Unit Converter
          </Button>

          <Button
            variant="outlined"
            size="small"
            startIcon={<ExportIcon />}
            onClick={handleOpenExport}
            sx={activeSidebar === 'export' ? toolbarButtonStyles.active : toolbarButtonStyles.inactive}
          >
            Export
          </Button>

          <Box
            sx={{
              flexGrow: 1,
              ...(isDetachedWindow && {
                cursor: 'grab',
                '&:active': {
                  cursor: 'grabbing'
                }
              })
            }}
            {...(isDetachedWindow && { 'data-tauri-drag-region': true })}
          />
        </Toolbar>
      </Paper>

      {/* Main Content Area */}
      <Box sx={{ display: 'flex', flex: 1, overflow: 'hidden', gap: 1 }}>
        {/* Spreadsheet Grid Container */}
        <Paper sx={{
          flex: 1,
          display: 'flex',
          flexDirection: 'column',
          overflow: 'hidden',
          minHeight: 0
        }}>

          {/* Grid Content Area */}
          <Box sx={{
            flex: 1,
            overflow: 'hidden',
            minHeight: 0,
            display: 'flex',
            position: 'relative',
            '& .dsg-container': {
              height: '100%',
              fontFamily: 'monospace',
            },
            '& .dsg-cell': {
              fontSize: '14px',
            },
            '& .dsg-cell-header': {
              backgroundColor: '#f5f5f5',
              fontWeight: 'bold',
            }
          }}>
            <Box sx={{ flex: 1, overflow: 'hidden' }}>
              <UniverAdapter
                ref={spreadsheetRef}
                initialData={spreadsheetData}
                onCellChange={handleCellChange}
                onFormulaIntercept={handleFormulaIntercept}
                onSelectionChange={handleSelectionChange}
                tabId={tabId}
              />
            </Box>
            {/* Uncertainty Propagation Sidebar - positioned within spreadsheet */}
            {activeSidebar === 'uncertainty' && (
              <UncertaintySidebar
                open={true}
                onClose={() => setActiveSidebar(null)}
                univerRef={spreadsheetRef}
                onSelectionChange={handleSelectionChange}
                onPropagationComplete={(resultRange: string) => {
                  console.log('Propagation complete, results in:', resultRange);
                  // Could refresh spreadsheet or show notification here
                }}
                variables={sidebarState.uncertaintyVariables}
                setVariables={(variables) => updateSidebarState('uncertaintyVariables', variables)}
                formula={sidebarState.uncertaintyFormula}
                setFormula={(formula) => updateSidebarState('uncertaintyFormula', formula)}
                outputValueRange={sidebarState.uncertaintyOutputValueRange}
                setOutputValueRange={(range) => updateSidebarState('uncertaintyOutputValueRange', range)}
                outputUncertaintyRange={sidebarState.uncertaintyOutputUncertaintyRange}
                setOutputUncertaintyRange={(range) => updateSidebarState('uncertaintyOutputUncertaintyRange', range)}
                outputConfidence={sidebarState.uncertaintyOutputConfidence}
                setOutputConfidence={(confidence) => updateSidebarState('uncertaintyOutputConfidence', confidence)}
              />
            )}
            {/* Unit Conversion Sidebar - positioned within spreadsheet */}
            {activeSidebar === 'unitConvert' && (
              <UnitConversionSidebar
                open={true}
                onClose={() => setActiveSidebar(null)}
                univerRef={spreadsheetRef}
                onSelectionChange={handleSelectionChange}
                category={sidebarState.unitConversionCategory}
                setCategory={(category) => updateSidebarState('unitConversionCategory', category)}
                fromUnit={sidebarState.unitConversionFromUnit}
                setFromUnit={(unit) => updateSidebarState('unitConversionFromUnit', unit)}
                toUnit={sidebarState.unitConversionToUnit}
                setToUnit={(unit) => updateSidebarState('unitConversionToUnit', unit)}
                value={sidebarState.unitConversionValue}
                setValue={(value) => updateSidebarState('unitConversionValue', value)}
              />
            )}
            {/* Quick Plot Sidebar - positioned within spreadsheet */}
            {activeSidebar === 'quickPlot' && (
              <QuickPlotSidebar
                open={true}
                onClose={() => setActiveSidebar(null)}
                univerRef={spreadsheetRef}
                onSelectionChange={handleSelectionChange}
                xRange={sidebarState.quickPlotXRange}
                setXRange={(range) => updateSidebarState('quickPlotXRange', range)}
                yRange={sidebarState.quickPlotYRange}
                setYRange={(range) => updateSidebarState('quickPlotYRange', range)}
                errorRange={sidebarState.quickPlotErrorRange}
                setErrorRange={(range) => updateSidebarState('quickPlotErrorRange', range)}
                xLabel={sidebarState.quickPlotXLabel}
                setXLabel={(label) => updateSidebarState('quickPlotXLabel', label)}
                yLabel={sidebarState.quickPlotYLabel}
                setYLabel={(label) => updateSidebarState('quickPlotYLabel', label)}
                plotType={sidebarState.quickPlotType}
                setPlotType={(type) => updateSidebarState('quickPlotType', type)}
                showErrorBars={sidebarState.quickPlotShowErrorBars}
                setShowErrorBars={(show) => updateSidebarState('quickPlotShowErrorBars', show)}
              />
            )}
            {/* Export Sidebar - positioned within spreadsheet */}
            {activeSidebar === 'export' && (
              <ExportSidebar
                open={true}
                onClose={() => setActiveSidebar(null)}
                univerRef={spreadsheetRef}
                onSelectionChange={handleSelectionChange}
                exportFormat={sidebarState.exportFormat}
                setExportFormat={(format) => updateSidebarState('exportFormat', format)}
                rangeMode={sidebarState.exportRangeMode}
                setRangeMode={(mode) => updateSidebarState('exportRangeMode', mode)}
                customRange={sidebarState.exportCustomRange}
                setCustomRange={(range) => updateSidebarState('exportCustomRange', range)}
                jsonFormat={sidebarState.exportJsonFormat}
                setJsonFormat={(format) => updateSidebarState('exportJsonFormat', format)}
                prettyPrint={sidebarState.exportPrettyPrint}
                setPrettyPrint={(pretty) => updateSidebarState('exportPrettyPrint', pretty)}
                customDelimiter={sidebarState.exportCustomDelimiter}
                setCustomDelimiter={(delimiter) => updateSidebarState('exportCustomDelimiter', delimiter)}
                getTrackedBounds={() => spreadsheetRef.current?.getTrackedBounds() ?? {}}
              />
            )}
          </Box>
        </Paper>
      </Box>
    </Box>
  );
};

export default SpreadsheetTab;
