import {
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  FormControl,
  InputLabel,
  MenuItem,
  Select,
  TextField,
} from '@mui/material';
import type { CsvImportSettings } from '../types/fittingTypes';

interface CsvSettingsDialogProps {
  open: boolean;
  settings: CsvImportSettings;
  onClose: () => void;
  onSettingsChange: (
    settings:
      | CsvImportSettings
      | ((prev: CsvImportSettings) => CsvImportSettings)
  ) => void;
}

const DROPDOWN_MAX_HEIGHT = 300;

const amberInputSx = {
  '& .MuiOutlinedInput-root': {
    backgroundColor: '#0f0f14',
    backgroundImage: 'none',
    '& fieldset': { borderColor: 'rgba(255,179,0,0.22)' },
    '&:hover fieldset': { borderColor: 'rgba(255,179,0,0.38)' },
    '&.Mui-focused fieldset': { borderColor: '#ffb300' },
  },
  '& .MuiInputLabel-root': { color: 'rgba(255,255,255,0.68)' },
  '& .MuiInputLabel-root.Mui-focused': { color: '#ffb300' },
  '& .MuiInputBase-input': { caretColor: '#ffb300' },
  '& .MuiSelect-icon': { color: 'rgba(255,179,0,0.8)' },
};

export default function CsvSettingsDialog({
  open,
  settings,
  onClose,
  onSettingsChange,
}: CsvSettingsDialogProps) {
  return (
    <Dialog
      open={open}
      onClose={onClose}
      maxWidth="xs"
      fullWidth
      slotProps={{
        paper: {
          sx: {
            backgroundColor: '#0f0f14 !important',
            backgroundImage: 'none !important',
            opacity: 1,
            backdropFilter: 'none !important',
            mixBlendMode: 'normal',
            border: '1px solid rgba(255,179,0,0.15)',
            boxShadow: '0 8px 24px rgba(0,0,0,0.6)',
          },
        },
      }}
    >
      <DialogTitle>Import Settings</DialogTitle>
      <DialogContent
        sx={{
          display: 'flex',
          flexDirection: 'column',
          gap: 2,
          pt: 2,
        }}
      >
        <FormControl size="small" fullWidth sx={[amberInputSx, { mt: 1 }]}>
          <InputLabel>Separator</InputLabel>
          <Select
            color="warning"
            value={settings.separator}
            label="Separator"
            MenuProps={{
              PaperProps: {
                sx: {
                  maxHeight: DROPDOWN_MAX_HEIGHT,
                  overflowY: 'auto',
                  backgroundColor: '#1a1a22 !important',
                  backgroundImage: 'none !important',
                  opacity: 1,
                  backdropFilter: 'none !important',
                  mixBlendMode: 'normal',
                  color: 'rgba(255,255,255,0.95)',
                  '& .MuiMenuItem-root': { color: 'inherit' },
                  '& .MuiMenuItem-root:hover': {
                    backgroundColor: 'rgba(255,179,0,0.06)',
                  },
                  '& .MuiMenuItem-root.Mui-selected, & .MuiMenuItem-root.Mui-selected:hover':
                    {
                      backgroundColor: 'rgba(255,179,0,0.12) !important',
                      color: 'inherit',
                    },
                },
              },
            }}
            onChange={(event) => {
              onSettingsChange((prev: CsvImportSettings) => ({
                ...prev,
                separator: event.target.value as ',' | ';' | '\t' | 'auto',
              }));
            }}
          >
            <MenuItem value="auto">Auto-detect</MenuItem>
            <MenuItem value=",">Comma (,)</MenuItem>
            <MenuItem value=";">Semicolon (;)</MenuItem>
            <MenuItem value={'\t'}>Tab</MenuItem>
          </Select>
        </FormControl>

        <FormControl size="small" fullWidth sx={amberInputSx}>
          <InputLabel>Decimal</InputLabel>
          <Select
            color="warning"
            value={settings.decimalFormat}
            label="Decimal"
            MenuProps={{
              PaperProps: {
                sx: {
                  maxHeight: DROPDOWN_MAX_HEIGHT,
                  overflowY: 'auto',
                  backgroundColor: '#1a1a22 !important',
                  backgroundImage: 'none !important',
                  opacity: 1,
                  backdropFilter: 'none !important',
                  mixBlendMode: 'normal',
                  color: 'rgba(255,255,255,0.95)',
                  '& .MuiMenuItem-root': { color: 'inherit' },
                  '& .MuiMenuItem-root:hover': {
                    backgroundColor: 'rgba(255,179,0,0.06)',
                  },
                  '& .MuiMenuItem-root.Mui-selected, & .MuiMenuItem-root.Mui-selected:hover':
                    {
                      backgroundColor: 'rgba(255,179,0,0.12) !important',
                      color: 'inherit',
                    },
                },
              },
            }}
            onChange={(event) => {
              onSettingsChange((prev: CsvImportSettings) => ({
                ...prev,
                decimalFormat: event.target.value as '.' | ',',
              }));
            }}
          >
            <MenuItem value=".">Point (.)</MenuItem>
            <MenuItem value=",">Comma (,)</MenuItem>
          </Select>
        </FormControl>

        <TextField
          color="warning"
          size="small"
          type="number"
          label="Skip rows"
          value={settings.skipRows}
          sx={amberInputSx}
          onChange={(event) => {
            onSettingsChange((prev: CsvImportSettings) => ({
              ...prev,
              skipRows: Math.max(0, Number(event.target.value)),
            }));
          }}
        />
      </DialogContent>
      <DialogActions>
        <Button color="warning" onClick={onClose}>
          Close
        </Button>
      </DialogActions>
    </Dialog>
  );
}
