import React from 'react';
import { Box } from '@mui/material';

interface DetachedTabWindowProps {
  children: React.ReactNode;
}

export function DetachedTabWindow({ children }: DetachedTabWindowProps) {
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
      backgroundColor: '#0a0a0a',
    }}>
      {/* Top bar is provided by `CustomTitleBar` from App.tsx; removed duplicate AppBar here */}

      {/* Content Area */}
      <Box sx={{
        flex: 1,
        overflow: 'auto',
        backgroundColor: '#0a0a0a',
      }}>
        {children}
      </Box>
    </Box>
  );
}
