import { createRoot } from 'react-dom/client';
import { ThemeProvider, CssBaseline, Box, Typography, Paper } from '@mui/material';
import { createAnafisTheme } from '@/tabs/spreadsheet/components/sidebar/themes';
import CustomTitleBar from '@/shared/components/CustomTitleBar';

const theme = createAnafisTheme();

function SettingsWindow() {
  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Box sx={{
        width: '650px',
        height: '700px',
        backgroundColor: 'background.default',
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
      }}>
        <CustomTitleBar title="AnaFis Settings" />
        
        <Box sx={{ 
          flex: 1, 
          display: 'flex', 
          alignItems: 'center', 
          justifyContent: 'center',
          p: 4
        }}>
          <Paper sx={{ 
            p: 4, 
            textAlign: 'center',
            maxWidth: 400,
            bgcolor: 'background.paper',
            border: '1px solid',
            borderColor: 'divider'
          }}>
            <Typography variant="h5" gutterBottom sx={{ color: 'text.primary', mb: 2 }}>
              Settings Placeholder
            </Typography>
            <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
              This is a placeholder for the settings window.
            </Typography>
            <Typography variant="body2" color="text.secondary">
              Settings functionality will be implemented in a future version.
            </Typography>
          </Paper>
        </Box>
      </Box>
    </ThemeProvider>
  );
}

// Auto-render immediately when this module loads
// This component auto-renders for standalone window usage (e.g., Tauri/Electron popup)
// Guards prevent rendering in test environments or during HMR to avoid side effects

const renderSettingsWindow = () => {
  // Skip rendering in test environment
  if (process.env.NODE_ENV === 'test') {
    return;
  }

  const container = document.getElementById('root');
  if (container) {
    try {
      const root = createRoot(container);
      root.render(<SettingsWindow />);
    } catch {
      // SettingsWindow: Error rendering
    }
  } else {
    // SettingsWindow: Root container not found
  }
};

// Try to render immediately
if (document.readyState === 'complete') {
  renderSettingsWindow();
} else {
  // Wait for DOM to be ready
  window.addEventListener('load', renderSettingsWindow);
}

export default SettingsWindow;
