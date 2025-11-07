// Tab color configuration - centralized for consistency
import React from 'react';
import HomeIcon from '@mui/icons-material/Home';
import TableChartIcon from '@mui/icons-material/TableChart';
import TrendingUpIcon from '@mui/icons-material/TrendingUp';
import CalculateIcon from '@mui/icons-material/Calculate';
import CasinoIcon from '@mui/icons-material/Casino';

// Color scheme type
type ColorScheme = {
  primary: string;
  secondary: string;
  accent: string;
  icon: string;
};

// Pre-defined color schemes for better performance
const COLOR_SCHEMES: Record<string, ColorScheme> = {
  home: {
    primary: '#9c27b0',
    secondary: '#9c27b0',
    accent: '#9c27b0',
    icon: '#9c27b0'
  },
  spreadsheet: {
    primary: '#2196f3',
    secondary: '#64b5f6',
    accent: '#64b5f6',
    icon: '#64b5f6'
  },
  fitting: {
    primary: '#ff9800',
    secondary: '#ffb74d',
    accent: '#ffb74d',
    icon: '#ffb74d'
  },
  solver: {
    primary: '#4caf50',
    secondary: '#81c784',
    accent: '#81c784',
    icon: '#81c784'
  },
  montecarlo: {
    primary: '#e91e63',
    secondary: '#f06292',
    accent: '#f06292',
    icon: '#f06292'
  },
  settings: {
    primary: '#9c27b0',
    secondary: '#ba68c8',
    accent: '#ba68c8',
    icon: '#ba68c8'
  },
  default: {
    primary: '#9c27b0',
    secondary: '#ba68c8',
    accent: '#ba68c8',
    icon: '#ba68c8'
  }
};

// Memoization cache for tab colors
// Using Map instead of WeakMap because tabId is a string, not an object reference
const colorCache = new Map<string, ColorScheme>();

// Maximum cache size to prevent unbounded growth
const MAX_CACHE_SIZE = 100;

/**
 * Bounded cache setter with FIFO eviction
 * Removes oldest inserted entry when cache reaches maximum size and adding a new key
 */
const setColorCache = (tabId: string, scheme: ColorScheme) => {
  // Only evict oldest entry if cache is at maximum size AND we're adding a new key
  if (colorCache.size >= MAX_CACHE_SIZE && !colorCache.has(tabId)) {
    const oldestKey = colorCache.keys().next().value;
    if (oldestKey) {
      colorCache.delete(oldestKey);
    }
  }
  colorCache.set(tabId, scheme);
};

export const getTabColors = (tabId: string) => {
  // Check cache first
  if (colorCache.has(tabId)) {
    return colorCache.get(tabId)!;
  }

  let colors: ColorScheme;
  if (tabId === 'home') {
    colors = COLOR_SCHEMES['home']!;
  } else if (tabId.includes('spreadsheet')) {
    colors = COLOR_SCHEMES['spreadsheet']!;
  } else if (tabId.includes('fitting')) {
    colors = COLOR_SCHEMES['fitting']!;
  } else if (tabId.includes('solver')) {
    colors = COLOR_SCHEMES['solver']!;
  } else if (tabId.includes('montecarlo')) {
    colors = COLOR_SCHEMES['montecarlo']!;
  } else if (tabId.includes('settings')) {
    colors = COLOR_SCHEMES['settings']!;
  } else {
    colors = COLOR_SCHEMES['default']!;
  }

  // Cache the result with bounded eviction
  setColorCache(tabId, colors);
  return colors;
};

// Tab icon configuration - centralized for consistency
export const getTabIcon = (tabId: string, fontSize: string = '1rem') => {
  const colors = getTabColors(tabId);

  if (tabId === 'home') {return React.createElement(HomeIcon, { sx: { fontSize, color: colors.icon } });}
  if (tabId.includes('optimized-spreadsheet')) {return React.createElement(TableChartIcon, { sx: { fontSize, color: colors.icon } });}
  if (tabId.includes('spreadsheet')) {return React.createElement(TableChartIcon, { sx: { fontSize, color: colors.icon } });}
  if (tabId.includes('fitting')) {return React.createElement(TrendingUpIcon, { sx: { fontSize, color: colors.icon } });}
  if (tabId.includes('solver')) {return React.createElement(CalculateIcon, { sx: { fontSize, color: colors.icon } });}
  if (tabId.includes('montecarlo')) {return React.createElement(CasinoIcon, { sx: { fontSize, color: colors.icon } });}
  return React.createElement(HomeIcon, { sx: { fontSize, color: colors.icon } });
};