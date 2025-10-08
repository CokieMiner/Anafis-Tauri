import React from 'react';
import { Box, useTheme } from '@mui/material';

interface DetachedTabWindowProps {
  children: React.ReactNode;
}

export function DetachedTabWindow({ children }: DetachedTabWindowProps) {
  const theme = useTheme();
  
  // Detached window actions (reattach/close) are provided by `CustomTitleBar`.

  return (
    <Box sx={{
      display: 'flex',
      flexDirection: 'column',
      height: '100%',
      width: '100%',
      margin: 0,
      padding: 0,
      overflow: 'hidden',
      backgroundColor: theme.palette.background.default,
    }}>
      {/* Top bar is provided by `CustomTitleBar` from App.tsx; removed duplicate AppBar here */}

      {/* Content Area */}
      <Box sx={{
        flex: 1,
        overflow: 'auto',
        backgroundColor: theme.palette.background.default,
      }}>
        {children}
      </Box>
    </Box>
  );
}
