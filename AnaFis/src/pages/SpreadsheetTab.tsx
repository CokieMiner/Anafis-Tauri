import React, { useState } from 'react';
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

const SpreadsheetTab: React.FC = () => {
  // Panel visibility
  const [showFunctionLibrary, setShowFunctionLibrary] = useState(false);

  // Workbook tabs state
  const [activeSheetTab, setActiveSheetTab] = useState(0);
  const [sheets] = useState(['Sheet1', 'Sheet2', 'Sheet3']); // Placeholder sheets

  // Active cell for formula bar
  const [activeCell] = useState<{ row: number; col: number } | null>({ row: 0, col: 0 });  // Handlers
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
        onFormulaSubmit={(_formula: string) => {
          // Handle formula submission
        }}
        onCancel={() => {
          // Handle formula cancellation
        }}
        onValueChange={(_value: string) => {
          // Handle value changes
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
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            backgroundColor: '#fafafa',
            minHeight: 0
          }}>
            <Typography variant="h6" color="text.secondary">
              Spreadsheet Grid Component
              <br />
              <Typography variant="body2" color="text.disabled">
                Grid implementation will be added here
              </Typography>
            </Typography>
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
              Cell: {activeCell ? `${String.fromCharCode(65 + activeCell.col)}${activeCell.row + 1}` : 'A1'}
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
