// Tab color configuration - centralized for consistency
import React from 'react';
import HomeIcon from '@mui/icons-material/Home';
import TableChartIcon from '@mui/icons-material/TableChart';
import TrendingUpIcon from '@mui/icons-material/TrendingUp';
import CalculateIcon from '@mui/icons-material/Calculate';
import CasinoIcon from '@mui/icons-material/Casino';

export const getTabColors = (tabId: string) => {
  if (tabId === 'home') {
    return {
      primary: '#9c27b0', // Purple
      secondary: '#ba68c8',
      accent: '#ba68c8',
      icon: '#ba68c8'
    };
  }
  if (tabId.includes('spreadsheet')) {
    return {
      primary: '#2196f3', // Light blue
      secondary: '#64b5f6',
      accent: '#64b5f6',
      icon: '#64b5f6'
    };
  }
  if (tabId.includes('fitting')) {
    return {
      primary: '#ff9800', // Orange
      secondary: '#ffb74d',
      accent: '#ffb74d',
      icon: '#ffb74d'
    };
  }
  if (tabId.includes('solver')) {
    return {
      primary: '#4caf50', // Green
      secondary: '#81c784',
      accent: '#81c784',
      icon: '#81c784'
    };
  }
  if (tabId.includes('montecarlo')) {
    return {
      primary: '#e91e63', // Pink
      secondary: '#f06292',
      accent: '#f06292',
      icon: '#f06292'
    };
  }
  if (tabId.includes('settings')) {
    return {
      primary: '#9c27b0', // Purple (same as home)
      secondary: '#ba68c8',
      accent: '#ba68c8',
      icon: '#ba68c8'
    };
  }
  return {
    primary: '#9c27b0', // Default to purple
    secondary: '#ba68c8',
    accent: '#ba68c8',
    icon: '#ba68c8'
  };
};

// Tab icon configuration - centralized for consistency
export const getTabIcon = (tabId: string, fontSize: string = '1rem') => {
  const colors = getTabColors(tabId);

  if (tabId === 'home') return React.createElement(HomeIcon, { sx: { fontSize, color: colors.icon } });
  if (tabId.includes('optimized-spreadsheet')) return React.createElement(TableChartIcon, { sx: { fontSize, color: colors.icon } });
  if (tabId.includes('spreadsheet')) return React.createElement(TableChartIcon, { sx: { fontSize, color: colors.icon } });
  if (tabId.includes('fitting')) return React.createElement(TrendingUpIcon, { sx: { fontSize, color: colors.icon } });
  if (tabId.includes('solver')) return React.createElement(CalculateIcon, { sx: { fontSize, color: colors.icon } });
  if (tabId.includes('montecarlo')) return React.createElement(CasinoIcon, { sx: { fontSize, color: colors.icon } });
  return React.createElement(HomeIcon, { sx: { fontSize, color: colors.icon } });
};