import React, { useState } from 'react';

// Material-UI Imports
import Box from '@mui/material/Box';
import Typography from '@mui/material/Typography';
import Tabs from '@mui/material/Tabs';
import Tab from '@mui/material/Tab';
import FormControl from '@mui/material/FormControl';
import InputLabel from '@mui/material/InputLabel';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import Switch from '@mui/material/Switch';
import FormControlLabel from '@mui/material/FormControlLabel';
import Button from '@mui/material/Button';
import Divider from '@mui/material/Divider';

interface SettingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

interface TabPanelProps {
  children?: React.ReactNode;
  index: number;
  value: number;
}

function TabPanel(props: TabPanelProps) {
  const { children, value, index, ...other } = props;

  return (
    <div
      role="tabpanel"
      hidden={value !== index}
      id={`settings-tabpanel-${index}`}
      aria-labelledby={`settings-tab-${index}`}
      {...other}
    >
      {value === index && (
        <Box sx={{ p: 2 }}>
          {children}
        </Box>
      )}
    </div>
  );
}

function a11yProps(index: number) {
  return {
    id: `settings-tab-${index}`,
    'aria-controls': `settings-tabpanel-${index}`,
  };
}

const SettingsDialog: React.FC<SettingsDialogProps> = ({ isOpen, onClose }) => {
  const [tabValue, setTabValue] = useState(0);
  const [language, setLanguage] = useState('en');
  const [autoUpdate, setAutoUpdate] = useState(true);
  const [updateFrequency, setUpdateFrequency] = useState('weekly');

  const handleTabChange = (_event: React.SyntheticEvent, newValue: number) => {
    setTabValue(newValue);
  };

  const handleLanguageChange = (event: any) => {
    setLanguage(event.target.value);
  };

  const handleAutoUpdateChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setAutoUpdate(event.target.checked);
  };

  const handleUpdateFrequencyChange = (event: any) => {
    setUpdateFrequency(event.target.value);
  };

  const handleSave = () => {
    // Here you would typically save the settings
    console.log('Settings saved:', { language, autoUpdate, updateFrequency });
    onClose();
  };

  const handleReset = () => {
    setLanguage('en');
    setAutoUpdate(true);
    setUpdateFrequency('weekly');
  };

  if (!isOpen) return null;

  return (
    <Box
      sx={{
        width: '100%',
        boxSizing: 'border-box',
        minWidth: '500px',
        maxWidth: '600px',
        '& *': { boxSizing: 'border-box' },
      }}
    >
      <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
        <Tabs value={tabValue} onChange={handleTabChange} aria-label="settings tabs">
          <Tab label="General" {...a11yProps(0)} />
          <Tab label="Updates" {...a11yProps(1)} />
        </Tabs>
      </Box>

      {/* General Tab */}
      <TabPanel value={tabValue} index={0}>
        <Typography variant="h6" gutterBottom>
          General Settings
        </Typography>

        <Box sx={{ mt: 2 }}>
          <FormControl fullWidth sx={{ mb: 3 }}>
            <InputLabel id="language-select-label">Language</InputLabel>
            <Select
              labelId="language-select-label"
              id="language-select"
              value={language}
              label="Language"
              onChange={handleLanguageChange}
            >
              <MenuItem value="en">English</MenuItem>
              <MenuItem value="pt">Português</MenuItem>
              <MenuItem value="es">Español</MenuItem>
              <MenuItem value="fr">Français</MenuItem>
              <MenuItem value="de">Deutsch</MenuItem>
            </Select>
          </FormControl>

          <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
            Choose your preferred language for the application interface.
          </Typography>
        </Box>
      </TabPanel>

      {/* Updates Tab */}
      <TabPanel value={tabValue} index={1}>
        <Typography variant="h6" gutterBottom>
          Update Settings
        </Typography>

        <Box sx={{ mt: 2 }}>
          <FormControlLabel
            control={
              <Switch
                checked={autoUpdate}
                onChange={handleAutoUpdateChange}
                color="primary"
              />
            }
            label="Enable automatic updates"
            sx={{ mb: 2 }}
          />

          <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
            When enabled, AnaFis will automatically check for and download updates.
          </Typography>

          <FormControl fullWidth sx={{ mb: 3 }} disabled={!autoUpdate}>
            <InputLabel id="update-frequency-label">Check for updates</InputLabel>
            <Select
              labelId="update-frequency-label"
              id="update-frequency-select"
              value={updateFrequency}
              label="Check for updates"
              onChange={handleUpdateFrequencyChange}
            >
              <MenuItem value="daily">Daily</MenuItem>
              <MenuItem value="weekly">Weekly</MenuItem>
              <MenuItem value="monthly">Monthly</MenuItem>
              <MenuItem value="manual">Manual only</MenuItem>
            </Select>
          </FormControl>

          <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
            How often should AnaFis check for available updates?
          </Typography>

          <Divider sx={{ my: 2 }} />

          <Box sx={{ display: 'flex', gap: 1, flexWrap: 'wrap' }}>
            <Button variant="outlined" size="small">
              Check for Updates Now
            </Button>
            <Button variant="outlined" size="small">
              View Update History
            </Button>
          </Box>
        </Box>
      </TabPanel>

      {/* Action Buttons */}
      <Box sx={{
        display: 'flex',
        justifyContent: 'flex-end',
        gap: 1,
        p: 2,
        borderTop: 1,
        borderColor: 'divider'
      }}>
        <Button onClick={handleReset} color="inherit">
          Reset to Defaults
        </Button>
        <Button onClick={onClose} color="inherit">
          Cancel
        </Button>
        <Button onClick={handleSave} variant="contained">
          Save Settings
        </Button>
      </Box>
    </Box>
  );
};

export default SettingsDialog;
