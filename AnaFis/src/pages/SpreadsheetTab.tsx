import React, { useState, useCallback, useEffect, useRef, useMemo } from 'react';
import {
  Box,
  Button,
  Toolbar,
  Paper
} from '@mui/material';
import {
  Transform as UnitConverterIcon,
  AutoFixHigh as UncertaintyIcon,
  BarChart as QuickPlotIcon,
  FileDownload as ExportIcon
} from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';
import { CUSTOM_FUNCTION_NAMES } from '../components/spreadsheet/univer/customFormulas';

import { UniverAdapter } from '../components/spreadsheet/univer';
import { SpreadsheetRef, CellValue, WorkbookData } from '../components/spreadsheet/SpreadsheetInterface';

import UncertaintySidebar from '../components/spreadsheet/UncertaintySidebar';
import UnitConversionSidebar from '../components/spreadsheet/UnitConversionSidebar';
import QuickPlotSidebar from '../components/spreadsheet/QuickPlotSidebar';
import ExportSidebar from '../components/spreadsheet/ExportSidebar';
import { ExportFormat, ExportRangeMode, JsonFormat } from '../types/export';
import { anafisColors } from '../themes';

interface Variable {
  name: string;
  valueRange: string;
  uncertaintyRange: string;
}

const SpreadsheetTab: React.FC = () => {
  // Sidebar state management
  type SidebarType = 'uncertainty' | 'unitConvert' | 'quickPlot' | 'export' | null;
  const [activeSidebar, setActiveSidebar] = useState<SidebarType>(null);
  
  // Uncertainty sidebar state - persisted across sidebar switches
  const [uncertaintyVariables, setUncertaintyVariables] = useState<Variable[]>([
    { name: 'a', valueRange: 'A1:A10', uncertaintyRange: 'B1:B10' }
  ]);
  const [uncertaintyFormula, setUncertaintyFormula] = useState<string>('');
  const [uncertaintyOutputValueRange, setUncertaintyOutputValueRange] = useState<string>('C1:C10');
  const [uncertaintyOutputUncertaintyRange, setUncertaintyOutputUncertaintyRange] = useState<string>('D1:D10');
  
  // Unit conversion sidebar state - persisted across sidebar switches
  const [unitConversionCategory, setUnitConversionCategory] = useState<string>('');
  const [unitConversionFromUnit, setUnitConversionFromUnit] = useState<string>('');
  const [unitConversionToUnit, setUnitConversionToUnit] = useState<string>('');
  const [unitConversionValue, setUnitConversionValue] = useState<string>('1');
  
  // Quick Plot sidebar state - persisted across sidebar switches
  const [quickPlotXRange, setQuickPlotXRange] = useState<string>('');
  const [quickPlotYRange, setQuickPlotYRange] = useState<string>('');
  const [quickPlotErrorRange, setQuickPlotErrorRange] = useState<string>('');
  const [quickPlotXLabel, setQuickPlotXLabel] = useState<string>('');
  const [quickPlotYLabel, setQuickPlotYLabel] = useState<string>('');
  const [quickPlotType, setQuickPlotType] = useState<'scatter' | 'line' | 'both'>('scatter');
  const [quickPlotShowErrorBars, setQuickPlotShowErrorBars] = useState<boolean>(false);
  
  // Export sidebar state - persisted across sidebar switches
  const [exportFormat, setExportFormat] = useState<ExportFormat>('csv');
  const [exportRangeMode, setExportRangeMode] = useState<ExportRangeMode>('selection');
  const [exportCustomRange, setExportCustomRange] = useState<string>('');
  const [exportIncludeHeaders, setExportIncludeHeaders] = useState<boolean>(true);
  const [exportJsonFormat, setExportJsonFormat] = useState<JsonFormat>('records');
  const [exportPrettyPrint, setExportPrettyPrint] = useState<boolean>(true);
  const [exportCustomDelimiter, setExportCustomDelimiter] = useState<string>('|');
  
  // Spreadsheet state
  const [spreadsheetData, setSpreadsheetData] = useState<WorkbookData | undefined>(undefined);
  const spreadsheetRef = useRef<SpreadsheetRef>(null);

  // Initialize empty workbook - memoized to prevent recreation
  const emptyWorkbook = useMemo((): WorkbookData => {
    const sheetId = 'sheet-01';
    
    return {
      id: 'spreadsheet-workbook',
      name: 'AnaFis Spreadsheet',
      appVersion: '1.0.0',
      locale: 'EN_US', // Abstract locale identifier
      styles: {},
      sheets: {
        [sheetId]: {
          id: sheetId,
          name: 'Sheet1',
          cellData: {}, // Start with empty cells
          rowCount: 1000,
          columnCount: 26,
        }
      },
      sheetOrder: [sheetId],
    };
  }, []); // Empty dependency array - only create once

  const createEmptyWorkbook = useCallback((): WorkbookData => {
    return emptyWorkbook;
  }, [emptyWorkbook]);

  const handleCellChange = useCallback((cellRef: string, value: CellValue) => {
    // Spreadsheet handles all data storage now - no backend sync needed
    console.log('Cell changed:', cellRef, value);
  }, []);  const handleFormulaIntercept = useCallback(async (cellRef: string, formula: string) => {
    console.log('Formula intercepted:', cellRef, formula);

    try {
      // Remove the '=' prefix if present
      const cleanFormula = formula.startsWith('=') ? formula.slice(1) : formula;

      // Check if formula contains our custom high-precision functions that need evaluation
      const hasCustomFunction = CUSTOM_FUNCTION_NAMES.some(func => cleanFormula.toUpperCase().includes(func));

      if (hasCustomFunction) {
        console.log('Formula contains custom function - letting Univer handle it directly');
        // Our functions are now registered with Univer, so let it handle them
        return;
      }

      // For complex formulas with variables or other functions, use our evaluation engine
      console.log('Complex formula - using high-precision evaluation');

      // Extract variable references from the formula (simple regex for cell references like A1, B2, etc.)
      const cellRefs = cleanFormula.match(/[A-Z]+\d+/g) || [];

      // Get unique cell references
      const uniqueRefs = [...new Set(cellRefs)];

      // Build variables map
      const variables: Record<string, number> = {};

      for (const ref of uniqueRefs) {
        const value = spreadsheetRef.current?.getCellValue(ref);
        if (typeof value === 'number') {
          variables[ref] = value;
        } else if (typeof value === 'string' && !isNaN(Number(value))) {
          variables[ref] = Number(value);
        } else {
          console.warn(`Could not get numeric value for cell ${ref}, got:`, value);
          // Skip this formula evaluation if we can't get all variables
          return;
        }
      }

      // Call Tauri command to evaluate the formula with high precision
      const result = await invoke<{
        value: number;
        success: boolean;
        error?: string;
      }>('evaluate_formula', {
        formula: cleanFormula,
        variables
      });

      if (result.success) {
        // Update the cell with the computed high-precision value
        spreadsheetRef.current?.updateCell(cellRef, { v: result.value });
        console.log(`High-precision formula evaluated: ${formula} = ${result.value}`);
      } else {
        console.error('High-precision formula evaluation failed:', result.error);
        // Update cell with error indicator
        spreadsheetRef.current?.updateCell(cellRef, { v: '#ERROR!' });
      }

    } catch (error) {
      console.error('Error in formula interception:', error);
    }
  }, []);

  const handleSelectionChange = useCallback((cellRef: string) => {
    // Selection change handling - pass to active sidebar
    
    // Call uncertainty sidebar handler if it exists
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    if (typeof (window as any).__uncertaintySidebarSelectionHandler === 'function') {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (window as any).__uncertaintySidebarSelectionHandler(cellRef);
    }
    
    // Call unit converter sidebar handler if it exists
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    if (typeof (window as any).__unitConverterSelectionHandler === 'function') {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (window as any).__unitConverterSelectionHandler(cellRef);
    }
    
    // Call quick plot sidebar handler if it exists
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    if (typeof (window as any).__quickPlotSelectionHandler === 'function') {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (window as any).__quickPlotSelectionHandler(cellRef);
    }
    
    // Call export sidebar handler if it exists
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    if (typeof (window as any).__exportSelectionHandler === 'function') {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (window as any).__exportSelectionHandler(cellRef);
    }
  }, []);

  useEffect(() => {
    // Initialize with empty spreadsheet - Univer handles all data
    const initializeSpreadsheet = () => {
      setSpreadsheetData(createEmptyWorkbook());
    };

    initializeSpreadsheet();
  }, [createEmptyWorkbook]);

  // Handlers
  const handleOpenUnitConverter = () => {
    // Toggle sidebar - if already open, close it; otherwise open it
    setActiveSidebar(prev => prev === 'unitConvert' ? null : 'unitConvert');
  };

  const handleOpenUncertaintyPropagation = () => {
    // Toggle sidebar - if already open, close it; otherwise open it
    setActiveSidebar(prev => prev === 'uncertainty' ? null : 'uncertainty');
  };

  const handleOpenQuickPlot = () => {
    // Toggle sidebar - if already open, close it; otherwise open it
    setActiveSidebar(prev => prev === 'quickPlot' ? null : 'quickPlot');
  };

  const handleOpenExport = () => {
    // Toggle sidebar - if already open, close it; otherwise open it
    setActiveSidebar(prev => prev === 'export' ? null : 'export');
  };

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Main Toolbar */}
      <Paper sx={{ mb: 1, bgcolor: '#0a0a0a' }}>
        <Toolbar variant="dense" sx={{ minHeight: 48 }}>
          <Button
            variant="outlined"
            size="small"
            startIcon={<QuickPlotIcon />}
            onClick={handleOpenQuickPlot}
            sx={{
              mr: 1,
              color: activeSidebar === 'quickPlot' ? anafisColors.spreadsheet : 'white',
              borderColor: activeSidebar === 'quickPlot' ? anafisColors.spreadsheet : '#64b5f6',
              backgroundColor: activeSidebar === 'quickPlot' ? 'rgba(33, 150, 243, 0.2)' : 'transparent',
              outline: 'none',
              '&:hover': {
                borderColor: anafisColors.spreadsheet,
                backgroundColor: activeSidebar === 'quickPlot' ? 'rgba(33, 150, 243, 0.3)' : 'rgba(33, 150, 243, 0.1)'
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
            }}
          >
            Quick Plot
          </Button>
          
          <Button
            variant="outlined"
            size="small"
            startIcon={<UncertaintyIcon />}
            onClick={handleOpenUncertaintyPropagation}
            sx={{
              mr: 1,
              color: activeSidebar === 'uncertainty' ? anafisColors.spreadsheet : 'white',
              borderColor: activeSidebar === 'uncertainty' ? anafisColors.spreadsheet : '#64b5f6',
              backgroundColor: activeSidebar === 'uncertainty' ? 'rgba(33, 150, 243, 0.2)' : 'transparent',
              outline: 'none',
              '&:hover': {
                borderColor: anafisColors.spreadsheet,
                backgroundColor: activeSidebar === 'uncertainty' ? 'rgba(33, 150, 243, 0.3)' : 'rgba(33, 150, 243, 0.1)'
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
            }}
          >
            Uncertainty Propagation
          </Button>

          <Button
            variant="outlined"
            size="small"
            startIcon={<UnitConverterIcon />}
            onClick={handleOpenUnitConverter}
            sx={{
              mr: 1,
              color: activeSidebar === 'unitConvert' ? anafisColors.spreadsheet : 'white',
              borderColor: activeSidebar === 'unitConvert' ? anafisColors.spreadsheet : '#64b5f6',
              backgroundColor: activeSidebar === 'unitConvert' ? 'rgba(33, 150, 243, 0.2)' : 'transparent',
              outline: 'none',
              '&:hover': {
                borderColor: anafisColors.spreadsheet,
                backgroundColor: activeSidebar === 'unitConvert' ? 'rgba(33, 150, 243, 0.3)' : 'rgba(33, 150, 243, 0.1)'
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
            }}
          >
            Unit Converter
          </Button>

          <Button
            variant="outlined"
            size="small"
            startIcon={<ExportIcon />}
            onClick={handleOpenExport}
            sx={{
              mr: 1,
              color: activeSidebar === 'export' ? anafisColors.spreadsheet : 'white',
              borderColor: activeSidebar === 'export' ? anafisColors.spreadsheet : '#64b5f6',
              backgroundColor: activeSidebar === 'export' ? 'rgba(33, 150, 243, 0.2)' : 'transparent',
              outline: 'none',
              '&:hover': {
                borderColor: anafisColors.spreadsheet,
                backgroundColor: activeSidebar === 'export' ? 'rgba(33, 150, 243, 0.3)' : 'rgba(33, 150, 243, 0.1)'
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
            }}
          >
            Export
          </Button>

          <Box sx={{ flexGrow: 1 }} />
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
              {spreadsheetData && (
                <UniverAdapter
                  ref={spreadsheetRef}
                  initialData={spreadsheetData}
                  onCellChange={handleCellChange}
                  onFormulaIntercept={handleFormulaIntercept}
                  onSelectionChange={handleSelectionChange}
                />
              )}
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
                variables={uncertaintyVariables}
                setVariables={setUncertaintyVariables}
                formula={uncertaintyFormula}
                setFormula={setUncertaintyFormula}
                outputValueRange={uncertaintyOutputValueRange}
                setOutputValueRange={setUncertaintyOutputValueRange}
                outputUncertaintyRange={uncertaintyOutputUncertaintyRange}
                setOutputUncertaintyRange={setUncertaintyOutputUncertaintyRange}
              />
            )}
            {/* Unit Conversion Sidebar - positioned within spreadsheet */}
            {activeSidebar === 'unitConvert' && (
              <UnitConversionSidebar
                open={true}
                onClose={() => setActiveSidebar(null)}
                univerRef={spreadsheetRef}
                onSelectionChange={handleSelectionChange}
                category={unitConversionCategory}
                setCategory={setUnitConversionCategory}
                fromUnit={unitConversionFromUnit}
                setFromUnit={setUnitConversionFromUnit}
                toUnit={unitConversionToUnit}
                setToUnit={setUnitConversionToUnit}
                value={unitConversionValue}
                setValue={setUnitConversionValue}
              />
            )}
            {/* Quick Plot Sidebar - positioned within spreadsheet */}
            {activeSidebar === 'quickPlot' && (
              <QuickPlotSidebar
                open={true}
                onClose={() => setActiveSidebar(null)}
                univerRef={spreadsheetRef}
                onSelectionChange={handleSelectionChange}
                xRange={quickPlotXRange}
                setXRange={setQuickPlotXRange}
                yRange={quickPlotYRange}
                setYRange={setQuickPlotYRange}
                errorRange={quickPlotErrorRange}
                setErrorRange={setQuickPlotErrorRange}
                xLabel={quickPlotXLabel}
                setXLabel={setQuickPlotXLabel}
                yLabel={quickPlotYLabel}
                setYLabel={setQuickPlotYLabel}
                plotType={quickPlotType}
                setPlotType={setQuickPlotType}
                showErrorBars={quickPlotShowErrorBars}
                setShowErrorBars={setQuickPlotShowErrorBars}
              />
            )}
            {/* Export Sidebar - positioned within spreadsheet */}
            {activeSidebar === 'export' && (
              <ExportSidebar
                open={true}
                onClose={() => setActiveSidebar(null)}
                univerRef={spreadsheetRef}
                onSelectionChange={handleSelectionChange}
                exportFormat={exportFormat}
                setExportFormat={setExportFormat}
                rangeMode={exportRangeMode}
                setRangeMode={setExportRangeMode}
                customRange={exportCustomRange}
                setCustomRange={setExportCustomRange}
                includeHeaders={exportIncludeHeaders}
                setIncludeHeaders={setExportIncludeHeaders}
                jsonFormat={exportJsonFormat}
                setJsonFormat={setExportJsonFormat}
                prettyPrint={exportPrettyPrint}
                setPrettyPrint={setExportPrettyPrint}
                customDelimiter={exportCustomDelimiter}
                setCustomDelimiter={setExportCustomDelimiter}
              />
            )}
          </Box>
        </Paper>

      </Box>
    </Box>
  );
};

export default SpreadsheetTab;
