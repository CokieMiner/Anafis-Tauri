import React, { useState } from 'react';
import {
  Box,
  Button,
  Toolbar,
  Paper
} from '@mui/material';
import {
  Transform as UnitConverterIcon,
  AutoFixHigh as UncertaintyIcon
} from '@mui/icons-material';
import { LocaleType, IWorkbookData, ICellData } from '@univerjs/core';
import UniverSpreadsheet, { UniverSpreadsheetRef } from '../components/spreadsheet/UniverSpreadsheet';
import UncertaintySidebar from '../components/spreadsheet/UncertaintySidebar';
import UnitConversionSidebar from '../components/spreadsheet/UnitConversionSidebar';

interface Variable {
  name: string;
  valueRange: string;
  uncertaintyRange: string;
}

const SpreadsheetTab: React.FC = () => {
  // Sidebar state management
  type SidebarType = 'uncertainty' | 'unitConvert' | null;
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
  
  // Spreadsheet state
  const [spreadsheetData, setSpreadsheetData] = useState<IWorkbookData | undefined>(undefined);
  const univerSpreadsheetRef = useRef<UniverSpreadsheetRef>(null);

  // Initialize empty Univer workbook
  const createEmptyWorkbook = (): IWorkbookData => {
    const sheetId = 'sheet-01';
    
    return {
      id: 'spreadsheet-workbook',
      name: 'AnaFis Spreadsheet',
      appVersion: '1.0.0',
      locale: LocaleType.EN_US,
      styles: {},
      sheets: {
        [sheetId]: {
          id: sheetId,
          name: 'Sheet1',
          cellData: {}, // Start with empty cells - Univer handles everything
          rowCount: 1000,
          columnCount: 26,
        }
      },
      sheetOrder: [sheetId],
    };
  };

  const handleCellChange = useCallback((cellRef: string, value: ICellData) => {
    // Univer handles all data storage now - no backend sync needed
    console.log('Cell changed:', cellRef, value);
  }, []);

  const handleFormulaIntercept = useCallback((cellRef: string, formula: string) => {
    // Univer handles formulas - no backend sync needed
    console.log('Formula set:', cellRef, formula);
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
  }, []);

  useEffect(() => {
    // Initialize with empty spreadsheet - Univer handles all data
    const initializeSpreadsheet = () => {
      setSpreadsheetData(createEmptyWorkbook());
    };

    initializeSpreadsheet();
  }, []);

  // Handlers
  const handleOpenUnitConverter = () => {
    // Toggle sidebar - if already open, close it; otherwise open it
    setActiveSidebar(prev => prev === 'unitConvert' ? null : 'unitConvert');
  };

  const handleOpenUncertaintyPropagation = () => {
    // Toggle sidebar - if already open, close it; otherwise open it
    setActiveSidebar(prev => prev === 'uncertainty' ? null : 'uncertainty');
  };

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Main Toolbar */}
      <Paper sx={{ mb: 1, bgcolor: '#0a0a0a' }}>
        <Toolbar variant="dense" sx={{ minHeight: 48 }}>
          <Button
            variant="outlined"
            size="small"
            startIcon={<UncertaintyIcon />}
            onClick={handleOpenUncertaintyPropagation}
            sx={{
              mr: 1,
              color: activeSidebar === 'uncertainty' ? '#2196f3' : 'white',
              borderColor: activeSidebar === 'uncertainty' ? '#2196f3' : '#64b5f6',
              backgroundColor: activeSidebar === 'uncertainty' ? 'rgba(33, 150, 243, 0.2)' : 'transparent',
              outline: 'none',
              '&:hover': {
                borderColor: '#2196f3',
                backgroundColor: activeSidebar === 'uncertainty' ? 'rgba(33, 150, 243, 0.3)' : 'rgba(33, 150, 243, 0.1)'
              },
              '&:focus': {
                borderColor: '#2196f3',
                outline: 'none',
              },
              '&:focus-visible': {
                borderColor: '#2196f3',
                outline: 'none',
                boxShadow: '0 0 0 2px rgba(33, 150, 243, 0.5)',
              },
              '&:active': {
                borderColor: '#2196f3',
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
              color: activeSidebar === 'unitConvert' ? '#2196f3' : 'white',
              borderColor: activeSidebar === 'unitConvert' ? '#2196f3' : '#64b5f6',
              backgroundColor: activeSidebar === 'unitConvert' ? 'rgba(33, 150, 243, 0.2)' : 'transparent',
              outline: 'none',
              '&:hover': {
                borderColor: '#2196f3',
                backgroundColor: activeSidebar === 'unitConvert' ? 'rgba(33, 150, 243, 0.3)' : 'rgba(33, 150, 243, 0.1)'
              },
              '&:focus': {
                borderColor: '#2196f3',
                outline: 'none',
              },
              '&:focus-visible': {
                borderColor: '#2196f3',
                outline: 'none',
                boxShadow: '0 0 0 2px rgba(33, 150, 243, 0.5)',
              },
              '&:active': {
                borderColor: '#2196f3',
                outline: 'none',
              }
            }}
          >
            Unit Converter
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
                <UniverSpreadsheet
                  ref={univerSpreadsheetRef}
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
                univerRef={univerSpreadsheetRef}
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
                univerRef={univerSpreadsheetRef}
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
          </Box>
        </Paper>

      </Box>
    </Box>
  );
};

export default SpreadsheetTab;
