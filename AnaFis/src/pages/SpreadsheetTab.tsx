import React, { useState, useEffect } from 'react';
import { DataSheetGrid } from '@sdziadkowiec/react-datasheet-grid';
import AutoSizer from 'react-virtualized-auto-sizer';
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
import FormulaBar from '../components/spreadsheet/FormulaBar.tsx';
import { FunctionLibrary } from '../components/spreadsheet/FunctionLibrary.tsx';
import { SpreadsheetData, CellType, Cell } from '../../types/spreadsheet.ts';
import { createUniversalColumn } from '../components/spreadsheet/UniversalColumn.tsx';
import { ContextMenu } from '../components/spreadsheet/ContextMenu.tsx';

const SpreadsheetTab: React.FC = () => {
  // Panel visibility
  const [showFunctionLibrary, setShowFunctionLibrary] = useState(false);

  // Workbook tabs state
  const [activeSheetTab, setActiveSheetTab] = useState(0);
  const [sheets] = useState(['Sheet1', 'Sheet2', 'Sheet3']); // Placeholder sheets

  // Active cell for formula bar
  const [activeCell, setActiveCell] = useState<{ row: number; col: number } | null>(null);

  const [data, setData] = useState<SpreadsheetData>([]);

  useEffect(() => {
    invoke<SpreadsheetData>('load_spreadsheet').then(setData);
  }, []);

  useEffect(() => {
    const interval = setInterval(() => {
      invoke('save_spreadsheet');
    }, 30000); // Auto-save every 30 seconds
    return () => clearInterval(interval);
  }, []);

  const handleCellUpdate = (rowIndex: number, columnIndex: number, newCellData: Cell) => {
    invoke('update_cell', { row: rowIndex, col: columnIndex, cell: newCellData });
  };

  const columns = [...Array(100)].map((_, i) => ({
    ...createUniversalColumn({ handleCellUpdate }),
    title: String.fromCharCode(65 + i),
    id: String.fromCharCode(65 + i),
  }));

  const handleCellTypeChange = (cell: { row: number; col: number }, type: CellType) => {
    const newData = [...data];
    const newCell: Cell = { type, value: null };
    // A more sophisticated implementation would try to convert the value
    newData[cell.row][cell.col] = newCell;
    setData(newData);
    handleCellUpdate(cell.row, cell.col, newCell);
  };

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
          >
            Functions
          </Button>

          <Button
            variant="outlined"
            size="small"
            startIcon={<UnitConverterIcon />}
            onClick={handleOpenUnitConverter}
          >
            Unit Converter
          </Button>

          <Box sx={{ flexGrow: 1 }} />
        </Toolbar>
      </Paper>

      {/* Formula Bar */}
      <FormulaBar
        activeCell={activeCell}
        onFormulaSubmit={(_formula: string) => {}}
        onCancel={() => {}}
        onValueChange={(_value: string) => {}}
      />

      {/* Workbook Tabs */}
      <Paper sx={{ mb: 1, bgcolor: '#0a0a0a' }}>
        <Tabs
          value={activeSheetTab}
          onChange={handleSheetTabChange}
          variant="scrollable"
          scrollButtons="auto"
        >
          {sheets.map((sheet, index) => (
            <Tab key={index} label={sheet} />
          ))}
          <Tab label="+" />
        </Tabs>
      </Paper>

      {/* Main Content Area */}
      <Box sx={{ display: 'flex', flex: 1, overflow: 'hidden', gap: 1 }}>
        {/* Spreadsheet Grid Container */}
        <Paper sx={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden', minHeight: 0 }}>
          {/* Grid Header */}
          <Box sx={{ p: 1, borderBottom: 1, borderColor: 'divider', backgroundColor: 'grey.50' }}>
            <Typography variant="subtitle2">
              Spreadsheet Grid - {sheets[activeSheetTab]}
            </Typography>
          </Box>

          {/* Grid Content Area */}
          <Box sx={{ flex: 1, minHeight: 0 }}>
            <AutoSizer>
              {({ height, width }) => (
                <DataSheetGrid
                  value={data}
                  onChange={setData}
                  columns={columns}
                  height={height}
                  width={width}
                  onActiveCellChange={setActiveCell}
                  contextMenuComponent={(props) => (
                    <ContextMenu
                      {...props}
                      onCellTypeChange={(type) =>
                        handleCellTypeChange(props.cursorIndex, type)
                      }
                    />
                  )}
                />
              )}
            </AutoSizer>
          </Box>
        </Paper>

        {/* Function Library Side Panel */}
        {showFunctionLibrary && (
          <Paper sx={{ width: 361, minWidth: 361, maxWidth: 361, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
            <FunctionLibrary
              onClose={() => setShowFunctionLibrary(false)}
              onFunctionSelect={(func: any) => {
                console.log('Function selected:', func);
              }}
            />
          </Paper>
        )}
      </Box>

      {/* Bottom Status Bar */}
      <Paper sx={{ mt: 1, p: 1, borderTop: 1, borderColor: 'divider', backgroundColor: '#0a0a0a' }}>
        <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          {/* Left side - Spreadsheet info */}
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
            <Typography variant="caption" sx={{ color: 'white' }}>
              Sheet: {sheets[activeSheetTab]}
            </Typography>
            <Divider orientation="vertical" flexItem sx={{ borderColor: 'rgba(255,255,255,0.3)' }} />
            <Typography variant="caption" sx={{ color: 'white' }}>
              Cell: {activeCell ? `${String.fromCharCode(65 + activeCell.col)}${activeCell.row + 1}` : 'A1'}
            </Typography>
            <Divider orientation="vertical" flexItem sx={{ borderColor: 'rgba(255,255,255,0.3)' }} />
            <Typography variant="caption" sx={{ color: 'white' }}>
              Ready
            </Typography>
          </Box>

          {/* Right side - System utilization */}
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <Chip icon={<MemoryIcon />} label="RAM: 45%" size="small" variant="outlined" />
            <Chip icon={<SpeedIcon />} label="CPU: 12%" size="small" variant="outlined" />
            <Chip icon={<StorageIcon />} label="Cache: 2.1MB" size="small" variant="outlined" />
          </Box>
        </Box>
      </Paper>
    </Box>
  );
};

export default SpreadsheetTab;
