import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import '@wasback/react-datasheet-grid/dist/style.css'
import App from './App.tsx'
import { ThemeProvider } from '@mui/material';
import CssBaseline from '@mui/material/CssBaseline';
import { createAnafisTheme } from './themes';

// Create theme using shared configuration
const theme = createAnafisTheme();

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <App />
    </ThemeProvider>
  </StrictMode>,
)
