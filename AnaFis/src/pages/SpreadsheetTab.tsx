import React, { useState, useEffect, useCallback, useRef } from 'react';
import {
  Box,
  Typography,
  Button,
  Toolbar,
  Paper,
  Tabs,
  Tab,
  Chip,
  Divider
} from '@mui/material';
import {
  Functions as FunctionsIcon,
  Transform as UnitConverterIcon,
  Memory as MemoryIcon,
  Speed as SpeedIcon,
  Storage as StorageIcon
} from '@mui/icons-material';
import { invoke } from '@tauri-apps/api/core';
import {
  DataSheetGrid,
  textColumn,
  keyColumn,
  Column,
  DataSheetGridRef
} from 'react-datasheet-grid';
import 'react-datasheet-grid/dist/style.css';
import FormulaBar from '../components/spreadsheet/FormulaBar.tsx';
import { FunctionLibrary } from '../components/spreadsheet/FunctionLibrary.tsx';


// Types for spreadsheet data
interface SpreadsheetRow {
  A?: string | null;
  B?: string | null;
  C?: string | null;
  D?: string | null;
  E?: string | null;
  F?: string | null;
  G?: string | null;
  H?: string | null;
  I?: string | null;
  J?: string | null;
}

interface CellReference {
  row: number;
  col: number;
}

const SpreadsheetTab: React.FC = () => {
  // Panel visibility
  const [showFunctionLibrary, setShowFunctionLibrary] = useState(false);

  // Workbook tabs state
  const [activeSheetTab, setActiveSheetTab] = useState(0);
  const [sheets] = useState(['Sheet1', 'Sheet2', 'Sheet3']); // Placeholder sheets

  // Spreadsheet data and state
  const [data, setData] = useState<SpreadsheetRow[]>(() =>
    Array.from({ length: 100 }, () => ({}))
  );
  const [activeCell, setActiveCell] = useState<CellReference>({ row: 0, col: 0 });
  const [selection, setSelection] = useState<{ min: CellReference; max: CellReference } | null>(null);

  // Ref for programmatic grid control
  const gridRef = useRef<DataSheetGridRef>(null);



  // Define columns for the spreadsheet with adaptive uncertainty support
  const columns: Column<SpreadsheetRow>[] = [
    { 
      ...keyColumn('A', textColumn as any), 
      title: 'A', 
      width: 120 
    },
    { 
      ...keyColumn('B', textColumn as any), 
      title: 'B', 
      width: 120 
    },
    { 
      ...keyColumn('C', textColumn as any), 
      title: 'C', 
      width: 120 
    },
    { 
      ...keyColumn('D', textColumn as any), 
      title: 'D', 
      width: 120 
    },
    { 
      ...keyColumn('E', textColumn as any), 
      title: 'E', 
      width: 120 
    },
    { 
      ...keyColumn('F', textColumn as any), 
      title: 'F', 
      width: 120 
    },
    { 
      ...keyColumn('G', textColumn as any), 
      title: 'G', 
      width: 120 
    },
    { 
      ...keyColumn('H', textColumn as any), 
      title: 'H', 
      width: 120 
    },
    { 
      ...keyColumn('I', textColumn as any), 
      title: 'I', 
      width: 120 
    },
    { 
      ...keyColumn('J', textColumn as any), 
      title: 'J', 
      width: 120 
    },
  ];

  // Common function to update active cell
  const updateActiveCell = useCallback((row: number, col: number) => {
    setActiveCell({ row, col });

    // Sync with backend
    invoke('set_spreadsheet_active_cell', { row, col })
      .catch(error => console.error('Failed to sync active cell:', error));
  }, []);

  // Handle active cell changes (for single cell clicks)
  const handleActiveCellChange = useCallback((opts: { cell: any | null }) => {
    const cell = opts.cell;
    if (cell && typeof cell.row === 'number' && typeof cell.col === 'number') {
      updateActiveCell(cell.row, cell.col);
    }
  }, [updateActiveCell]);

  // Handle selection changes (for range selections, fallback)
  const handleSelectionChange = useCallback((selection: any) => {
    if (selection && selection.min) {
      // Update active cell to top-left of selection
      updateActiveCell(selection.min.row, selection.min.col);

      // Store selection for range display
      if (selection.max && (selection.min.row !== selection.max.row || selection.min.col !== selection.max.col)) {
        // Multi-cell selection
        setSelection({
          min: { row: selection.min.row, col: selection.min.col },
          max: { row: selection.max.row, col: selection.max.col }
        });
      } else {
        // Single cell selection
        setSelection(null);
      }
    }
  }, [updateActiveCell]);

  // Get current cell value for formula bar
  const getCurrentCellValue = useCallback(() => {
    const columnKey = String.fromCharCode(65 + activeCell.col) as keyof SpreadsheetRow;
    const value = data[activeCell.row]?.[columnKey] || '';
    return value;
  }, [data, activeCell]);

  // Get cell reference display (single cell or range)
  const getCellReferenceDisplay = useCallback(() => {
    if (selection) {
      // Multi-cell selection - show range
      const minCol = String.fromCharCode(65 + selection.min.col);
      const maxCol = String.fromCharCode(65 + selection.max.col);
      const minRow = selection.min.row + 1;
      const maxRow = selection.max.row + 1;
      return `${minCol}${minRow}:${maxCol}${maxRow}`;
    } else {
      // Single cell selection
      return `${String.fromCharCode(65 + activeCell.col)}${activeCell.row + 1}`;
    }
  }, [activeCell, selection]);



  // Fill multiple cells with the same value
  const fillSelectedCells = useCallback((value: string) => {
    if (!selection) {
      // Single cell - just update normally
      const columnKey = String.fromCharCode(65 + activeCell.col) as keyof SpreadsheetRow;
      const newData = [...data];
      if (!newData[activeCell.row]) {
        newData[activeCell.row] = {};
      }
      newData[activeCell.row][columnKey] = value;
      setData(newData);

      // Sync with backend
      const cellRef = `${columnKey}${activeCell.row + 1}`;
      invoke('set_spreadsheet_cell_value', { cellRef, content: value })
        .catch(error => console.error('Failed to sync cell value:', error));
      return;
    }

    // Multi-cell selection - fill all cells in range
    const newData = [...data];

    for (let row = selection.min.row; row <= selection.max.row; row++) {
      for (let col = selection.min.col; col <= selection.max.col; col++) {
        const columnKey = String.fromCharCode(65 + col) as keyof SpreadsheetRow;

        // Ensure row exists
        if (!newData[row]) {
          newData[row] = {};
        }

        // Set cell value
        newData[row][columnKey] = value;

        // Sync with backend
        const cellRef = `${String.fromCharCode(65 + col)}${row + 1}`;
        invoke('set_spreadsheet_cell_value', { cellRef, content: value })
          .catch(error => console.error('Failed to sync cell value:', error));
      }
    }

    setData(newData);
  }, [data, activeCell, selection]);



  // Handle cell value changes
  const handleCellChange = useCallback(async (newData: SpreadsheetRow[], operations: any[]) => {
    setData(newData);

    // Sync changes with backend
    for (const operation of operations) {
      if (operation.type === 'UPDATE') {
        const { fromRowIndex, columnId, value } = operation;
        const cellRef = `${columnId}${fromRowIndex + 1}`;

        try {
          await invoke('set_spreadsheet_cell_value', {
            cellRef,
            content: value || ''
          });
        } catch (error) {
          console.error('Failed to sync cell value:', error);
        }
      }
    }
  }, []);

  // Load initial data from backend
  useEffect(() => {
    const loadSpreadsheetState = async () => {
      try {
        const state = await invoke('get_spreadsheet_state');
        console.log('Loaded spreadsheet state:', state);
        // TODO: Convert backend state to grid data format
      } catch (error) {
        console.error('Failed to load spreadsheet state:', error);
      }
    };

    loadSpreadsheetState();
  }, []);

  // Handlers
  const handleOpenFunctionLibrary = () => {
    setShowFunctionLibrary(true);
  };

  const handleOpenUnitConverter = async () => {
    try {
      await invoke('open_unit_conversion_window');
    } catch (error) {
      console.error('Failed to open unit conversion window:', error);
    }
  };

  const handleSheetTabChange = (_event: React.SyntheticEvent, newValue: number) => {
    setActiveSheetTab(newValue);
  };

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Main Toolbar */}
      <Paper sx={{ mb: 1, bgcolor: '#0a0a0a' }}>
        <Toolbar variant="dense" sx={{ minHeight: 48 }}>
          <Button
            variant="outlined"
            size="small"
            startIcon={<FunctionsIcon />}
            onClick={handleOpenFunctionLibrary}
            sx={{
              mr: 1,
              color: 'white',
              borderColor: '#64b5f6',
              '&:hover': {
                borderColor: '#42a5f5',
                backgroundColor: 'rgba(100, 181, 246, 0.1)'
              }
            }}
          >
            Functions
          </Button>

          <Button
            variant="outlined"
            size="small"
            startIcon={<UnitConverterIcon />}
            onClick={handleOpenUnitConverter}
            sx={{
              mr: 1,
              color: 'white',
              borderColor: '#64b5f6',
              '&:hover': {
                borderColor: '#42a5f5',
                backgroundColor: 'rgba(100, 181, 246, 0.1)'
              }
            }}
          >
            Unit Converter
          </Button>

          <Box sx={{ flexGrow: 1 }} />
        </Toolbar>
      </Paper>

      {/* Formula Bar */}
      <FormulaBar
        activeCell={activeCell}
        currentValue={getCurrentCellValue()}
        cellReference={getCellReferenceDisplay()}
        onFormulaSubmit={(formula: string) => {
          // Handle formula submission (single cell only)
          const columnKey = String.fromCharCode(65 + activeCell.col) as keyof SpreadsheetRow;
          const newData = [...data];
          if (!newData[activeCell.row]) {
            newData[activeCell.row] = {};
          }
          newData[activeCell.row][columnKey] = formula;
          setData(newData);

          // Sync with backend
          const cellRef = `${columnKey}${activeCell.row + 1}`;
          invoke('set_spreadsheet_cell_value', { cellRef, content: formula })
            .catch(error => console.error('Failed to sync formula:', error));
        }}
        onMultiCellFill={(value: string) => {
          // Handle multi-cell fill (Ctrl+Enter or Ctrl+Shift+Enter)
          fillSelectedCells(value);
        }}
        onCancel={() => {
          // Handle formula cancellation - could revert to previous value
        }}
        onValueChange={(value: string) => {
          // Handle value changes in real-time (only for active cell)
          const columnKey = String.fromCharCode(65 + activeCell.col) as keyof SpreadsheetRow;
          const newData = [...data];
          if (!newData[activeCell.row]) {
            newData[activeCell.row] = {};
          }
          newData[activeCell.row][columnKey] = value;
          setData(newData);
        }}

      />      {/* Workbook Tabs */}
      <Paper sx={{ mb: 1, bgcolor: '#0a0a0a' }}>
        <Tabs
          value={activeSheetTab}
          onChange={handleSheetTabChange}
          variant="scrollable"
          scrollButtons="auto"
          sx={{
            minHeight: 36,
            '& .MuiTab-root': {
              color: 'white',
              '&.Mui-selected': {
                color: '#64b5f6'
              }
            },
            '& .MuiTabs-indicator': {
              backgroundColor: '#64b5f6'
            }
          }}
        >
          {sheets.map((sheet, index) => (
            <Tab
              key={index}
              label={sheet}
              sx={{ minHeight: 36, fontSize: '0.875rem' }}
            />
          ))}
          <Tab
            label="+"
            sx={{
              minHeight: 36,
              minWidth: 40,
              fontSize: '1rem',
              fontWeight: 'bold'
            }}
          />
        </Tabs>
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
          {/* Grid Header */}
          <Box sx={{
            p: 1,
            borderBottom: 1,
            borderColor: 'divider',
            backgroundColor: 'grey.50'
          }}>
            <Typography variant="subtitle2">
              Spreadsheet Grid - {sheets[activeSheetTab]}
            </Typography>
          </Box>

          {/* Grid Content Area */}
          <Box sx={{
            flex: 1,
            overflow: 'hidden',
            minHeight: 0,
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
            <DataSheetGrid
              ref={gridRef}
              value={data}
              onChange={handleCellChange}
              columns={columns}
              height={600}
              rowHeight={30}
              headerRowHeight={35}
              addRowsComponent={false}
              onActiveCellChange={handleActiveCellChange}
              onSelectionChange={handleSelectionChange}
            />
          </Box>
        </Paper>

        {/* Function Library Side Panel */}
        {showFunctionLibrary && (
          <Paper sx={{
            width: 361,
            minWidth: 361,
            maxWidth: 361,
            display: 'flex',
            flexDirection: 'column',
            overflow: 'hidden'
          }}>
            <FunctionLibrary
              onClose={() => setShowFunctionLibrary(false)}
              onFunctionSelect={(func: any) => {
                console.log('Function selected:', func);
                // Function will be inserted into formula bar in future
              }}
            />
          </Paper>
        )}


      </Box>

      {/* Bottom Status Bar */}
      <Paper sx={{
        mt: 1,
        p: 1,
        borderTop: 1,
        borderColor: 'divider',
        backgroundColor: '#0a0a0a'
      }}>
        <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          {/* Left side - Spreadsheet info */}
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
            <Typography variant="caption" sx={{ color: 'white' }}>
              Sheet: {sheets[activeSheetTab]}
            </Typography>
            <Divider orientation="vertical" flexItem sx={{ borderColor: 'rgba(255,255,255,0.3)' }} />
            <Typography variant="caption" sx={{ color: 'white' }}>
              Cell: {getCellReferenceDisplay()}
            </Typography>
            <Divider orientation="vertical" flexItem sx={{ borderColor: 'rgba(255,255,255,0.3)' }} />
            <Typography variant="caption" sx={{ color: 'white' }}>
              Ready
            </Typography>
          </Box>

          {/* Right side - System utilization */}
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <Chip
              icon={<MemoryIcon sx={{ color: 'white' }} />}
              label="RAM: 45%"
              size="small"
              variant="outlined"
              sx={{
                color: 'white',
                borderColor: '#64b5f6',
                '& .MuiChip-label': { color: 'white' }
              }}
            />
            <Chip
              icon={<SpeedIcon sx={{ color: 'white' }} />}
              label="CPU: 12%"
              size="small"
              variant="outlined"
              sx={{
                color: 'white',
                borderColor: '#64b5f6',
                '& .MuiChip-label': { color: 'white' }
              }}
            />
            <Chip
              icon={<StorageIcon sx={{ color: 'white' }} />}
              label="Cache: 2.1MB"
              size="small"
              variant="outlined"
              sx={{
                color: 'white',
                borderColor: '#64b5f6',
                '& .MuiChip-label': { color: 'white' }
              }}
            />
          </Box>
        </Box>
      </Paper>
    </Box>
  );
};

export default SpreadsheetTab;
