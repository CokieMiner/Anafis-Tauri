import React, { useMemo } from 'react';
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
  FileDownload as ExportIcon,
  FileUpload as ImportIcon,
  Assessment as StatisticsIcon
} from '@mui/icons-material';

import { anafisColors } from '@/tabs/spreadsheet/components/sidebar/themes';

type SidebarType = 'uncertainty' | 'unitConvert' | 'quickPlot' | 'export' | 'import' | 'statistics' | null;

interface SpreadsheetSidebarToolbarProps {
  activeSidebar: SidebarType;
  onSidebarToggle: (sidebar: SidebarType) => void;
  isDetachedWindow?: boolean;
}

const SpreadsheetSidebarToolbar: React.FC<SpreadsheetSidebarToolbarProps> = ({
  activeSidebar,
  onSidebarToggle,
  isDetachedWindow = false
}) => {
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

  return (
    <Paper sx={{ mb: 1, bgcolor: '#0a0a0a' }}>
      <Toolbar variant="dense" sx={{ minHeight: 48 }}>
        <Button
          variant="outlined"
          size="small"
          startIcon={<ImportIcon />}
          onClick={() => onSidebarToggle(activeSidebar === 'import' ? null : 'import')}
          sx={activeSidebar === 'import' ? toolbarButtonStyles.active : toolbarButtonStyles.inactive}
        >
          Import
        </Button>

        <Button
          variant="outlined"
          size="small"
          startIcon={<UnitConverterIcon />}
          onClick={() => onSidebarToggle(activeSidebar === 'unitConvert' ? null : 'unitConvert')}
          sx={activeSidebar === 'unitConvert' ? toolbarButtonStyles.active : toolbarButtonStyles.inactive}
        >
          Unit Converter
        </Button>

        <Button
          variant="outlined"
          size="small"
          startIcon={<UncertaintyIcon />}
          onClick={() => onSidebarToggle(activeSidebar === 'uncertainty' ? null : 'uncertainty')}
          sx={activeSidebar === 'uncertainty' ? toolbarButtonStyles.active : toolbarButtonStyles.inactive}
        >
          Uncertainty Propagation
        </Button>

        <Button
          variant="outlined"
          size="small"
          startIcon={<QuickPlotIcon />}
          onClick={() => onSidebarToggle(activeSidebar === 'quickPlot' ? null : 'quickPlot')}
          sx={activeSidebar === 'quickPlot' ? toolbarButtonStyles.active : toolbarButtonStyles.inactive}
        >
          Quick Plot
        </Button>

        <Button
          variant="outlined"
          size="small"
          startIcon={<StatisticsIcon />}
          onClick={() => onSidebarToggle(activeSidebar === 'statistics' ? null : 'statistics')}
          sx={activeSidebar === 'statistics' ? toolbarButtonStyles.active : toolbarButtonStyles.inactive}
        >
          Statistics
        </Button>

        <Button
          variant="outlined"
          size="small"
          startIcon={<ExportIcon />}
          onClick={() => onSidebarToggle(activeSidebar === 'export' ? null : 'export')}
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
  );
};

export default SpreadsheetSidebarToolbar;
