import {
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
import { UnifiedButton } from '@/shared/components/UnifiedButton';
import { anafisTheme } from '@/shared/theme/unifiedTheme';
import type { CsvImportSettings } from '@/tabs/fitting/types/fittingTypes';

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
    backgroundColor: anafisTheme.colors.background.primary,
    backgroundImage: 'none',
    '& fieldset': { borderColor: `${anafisTheme.colors.tabs.fitting.main}38` },
    '&:hover fieldset': {
      borderColor: `${anafisTheme.colors.tabs.fitting.main}61`,
    },
    '&.Mui-focused fieldset': {
      borderColor: anafisTheme.colors.tabs.fitting.main,
    },
  },
  '& .MuiInputLabel-root': { color: anafisTheme.colors.text.tertiary },
  '& .MuiInputLabel-root.Mui-focused': {
    color: anafisTheme.colors.tabs.fitting.main,
  },
  '& .MuiInputBase-input': { caretColor: anafisTheme.colors.tabs.fitting.main },
  '& .MuiSelect-icon': { color: `${anafisTheme.colors.tabs.fitting.main}CC` },
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
            backgroundColor: `${anafisTheme.colors.background.primary} !important`,
            backgroundImage: 'none !important',
            opacity: 1,
            backdropFilter: 'none !important',
            mixBlendMode: 'normal',
            border: `1px solid ${anafisTheme.colors.tabs.fitting.main}26`,
            boxShadow: anafisTheme.shadows.xl,
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
              slotProps: {
                paper: {
                  sx: {
                    maxHeight: DROPDOWN_MAX_HEIGHT,
                    overflowY: 'auto',
                    backgroundColor: `${anafisTheme.colors.background.elevated} !important`,
                    backgroundImage: 'none !important',
                    opacity: 1,
                    backdropFilter: 'none !important',
                    mixBlendMode: 'normal',
                    color: anafisTheme.colors.text.primary,
                    '& .MuiMenuItem-root': { color: 'inherit' },
                    '& .MuiMenuItem-root:hover': {
                      backgroundColor: `${anafisTheme.colors.tabs.fitting.main}0F`,
                    },
                    '& .MuiMenuItem-root.Mui-selected, & .MuiMenuItem-root.Mui-selected:hover':
                      {
                        backgroundColor: `${anafisTheme.colors.tabs.fitting.main}1F !important`,
                        color: 'inherit',
                      },
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
              slotProps: {
                paper: {
                  sx: {
                    maxHeight: DROPDOWN_MAX_HEIGHT,
                    overflowY: 'auto',
                    backgroundColor: `${anafisTheme.colors.background.elevated} !important`,
                    backgroundImage: 'none !important',
                    opacity: 1,
                    backdropFilter: 'none !important',
                    mixBlendMode: 'normal',
                    color: anafisTheme.colors.text.primary,
                    '& .MuiMenuItem-root': { color: 'inherit' },
                    '& .MuiMenuItem-root:hover': {
                      backgroundColor: `${anafisTheme.colors.tabs.fitting.main}0F`,
                    },
                    '& .MuiMenuItem-root.Mui-selected, & .MuiMenuItem-root.Mui-selected:hover':
                      {
                        backgroundColor: `${anafisTheme.colors.tabs.fitting.main}1F !important`,
                        color: 'inherit',
                      },
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
        <UnifiedButton
          variant="secondary"
          onClick={onClose}
          sx={{
            color: anafisTheme.colors.tabs.fitting.main,
            borderColor: anafisTheme.colors.tabs.fitting.main,
            '&:hover': {
              borderColor: anafisTheme.colors.tabs.fitting.main,
              backgroundColor: `${anafisTheme.colors.tabs.fitting.main}1A`,
            },
          }}
        >
          Close
        </UnifiedButton>
      </DialogActions>
    </Dialog>
  );
}
