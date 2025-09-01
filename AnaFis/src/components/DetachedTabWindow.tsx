import React, { useState } from 'react';
import { Box, Button, Typography, AppBar, Toolbar } from '@mui/material';
import { ArrowBackIcon, CloseIcon, HomeIcon, TableChartIcon, TrendingUpIcon, CalculateIcon, CasinoIcon } from '../icons';
import { invoke } from '@tauri-apps/api/core';

interface DetachedTabWindowProps {
  tabId: string;
  tabTitle: string;
  children: React.ReactNode;
}

export function DetachedTabWindow({ tabId, tabTitle, children }: DetachedTabWindowProps) {
  const [isReattaching, setIsReattaching] = useState(false);

  // Get tab icon based on type
  const getTabIcon = () => {
    if (tabId === 'home') return <HomeIcon sx={{ fontSize: '1.2rem', mr: 1, color: '#ba68c8' }} />;
    if (tabId.includes('spreadsheet')) return <TableChartIcon sx={{ fontSize: '1.2rem', mr: 1, color: '#64b5f6' }} />;
    if (tabId.includes('fitting')) return <TrendingUpIcon sx={{ fontSize: '1.2rem', mr: 1, color: '#ffb74d' }} />;
    if (tabId.includes('solver')) return <CalculateIcon sx={{ fontSize: '1.2rem', mr: 1, color: '#81c784' }} />;
    if (tabId.includes('montecarlo')) return <CasinoIcon sx={{ fontSize: '1.2rem', mr: 1, color: '#f06292' }} />;
    return <HomeIcon sx={{ fontSize: '1.2rem', mr: 1, color: '#ba68c8' }} />;
  };

  const handleReattach = async () => {
    try {
      setIsReattaching(true);

      // Send tab back to main window
      await invoke('send_tab_to_main', {
        tabId,
        tabType: tabId.split('-')[0],
        tabTitle: decodeURIComponent(tabTitle)
      });

      // Close this detached window
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const currentWindow = getCurrentWindow();
      await currentWindow.close();
    } catch (error) {
      console.error('Failed to reattach tab:', error);
      setIsReattaching(false);
    }
  };

  const handleClose = async () => {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const currentWindow = getCurrentWindow();
      await currentWindow.close();
    } catch (error) {
      console.error('Failed to close window:', error);
    }
  };

  return (
    <Box sx={{
      display: 'flex',
      flexDirection: 'column',
      height: '100vh',
      width: '100vw',
      margin: 0,
      padding: 0,
      overflow: 'hidden',
      backgroundColor: '#0a0a0a',
    }}>
      {/* Custom Title Bar */}
      <AppBar
        position="static"
        sx={{
          background: 'linear-gradient(135deg, rgba(26, 26, 26, 0.95) 0%, rgba(42, 42, 42, 0.95) 100%)',
          backdropFilter: 'blur(20px)',
          borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
          boxShadow: '0 4px 20px rgba(0, 0, 0, 0.3)',
        }}
      >
        <Toolbar sx={{ gap: 1, minHeight: '48px' }}>
          {/* Reattach Button */}
          <Button
            onClick={handleReattach}
            disabled={isReattaching}
            startIcon={<ArrowBackIcon />}
            sx={{
              color: '#ffffff',
              background: 'linear-gradient(135deg, rgba(33, 150, 243, 0.1) 0%, rgba(33, 150, 243, 0.05) 100%)',
              border: '1px solid rgba(33, 150, 243, 0.3)',
              borderRadius: 2,
              px: 2,
              py: 0.8,
              fontWeight: 600,
              fontSize: '0.9rem',
              mr: 1,
              transition: 'all 0.25s cubic-bezier(.2,.8,.2,1)',
              boxShadow: '0 2px 8px rgba(33, 150, 243, 0.15)',
              '&:hover': {
                background: 'linear-gradient(135deg, rgba(33, 150, 243, 0.15) 0%, rgba(33, 150, 243, 0.08) 100%)',
                borderColor: 'rgba(33, 150, 243, 0.5)',
                transform: 'translateY(-1px)',
                boxShadow: '0 4px 16px rgba(33, 150, 243, 0.25)',
                color: '#2196f3',
              },
              '&:disabled': {
                opacity: 0.6,
                cursor: 'not-allowed',
              },
            }}
          >
            {isReattaching ? 'Reattaching...' : 'Reattach to Main'}
          </Button>

          {/* Tab Title */}
          <Box
            sx={{
              flex: 1,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            {getTabIcon()}
            <Typography
              variant="h6"
              sx={{
                color: '#ffffff',
                fontWeight: 600,
                fontSize: '1rem',
              }}
            >
              {tabTitle}
            </Typography>
          </Box>

          {/* Close Button */}
          <Button
            onClick={handleClose}
            startIcon={<CloseIcon />}
            sx={{
              color: '#ffffff',
              background: 'linear-gradient(135deg, rgba(244, 67, 54, 0.1) 0%, rgba(244, 67, 54, 0.05) 100%)',
              border: '1px solid rgba(244, 67, 54, 0.3)',
              borderRadius: 2,
              px: 2,
              py: 0.8,
              fontWeight: 600,
              fontSize: '0.9rem',
              transition: 'all 0.25s cubic-bezier(.2,.8,.2,1)',
              boxShadow: '0 2px 8px rgba(244, 67, 54, 0.15)',
              '&:hover': {
                background: 'linear-gradient(135deg, rgba(244, 67, 54, 0.15) 0%, rgba(244, 67, 54, 0.08) 100%)',
                borderColor: 'rgba(244, 67, 54, 0.5)',
                transform: 'translateY(-1px)',
                boxShadow: '0 4px 16px rgba(244, 67, 54, 0.25)',
                color: '#f44336',
              },
            }}
          >
            Close
          </Button>
        </Toolbar>
      </AppBar>

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
