// Persistent tab content renderer - keeps all tabs mounted but shows only active tab
import React, { Suspense } from 'react';
import { Box, CircularProgress, useTheme } from '@mui/material';
import { anafisColors } from '@/tabs/spreadsheet/components/sidebar/themes';
import type { Tab } from '@/core/types/tabs';

interface OptimizedTabRendererProps {
  tabs: Tab[];
  activeTabId: string | null;
}

// Loading fallback component
const TabLoadingFallback = () => {
  const theme = useTheme();
  return (
    <Box sx={{
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      height: '100%',
      width: '100%',
      backgroundColor: theme.palette.background.default
    }}>
      <CircularProgress sx={{ color: anafisColors.primary }} />
    </Box>
  );
};

// Memoized tab content wrapper - keeps all tabs mounted but shows only active tab
const TabContentWrapper = React.memo<{
  tab: Tab;
  isActive: boolean;
}>(({ tab, isActive }) => {
  const content = (
    <Suspense fallback={<TabLoadingFallback />}>
      {tab.content}
    </Suspense>
  );

  // Keeps all tabs mounted but shows only active tab (inactive tabs hidden), preserving state
  return (
    <Box sx={{
      width: '100%',
      height: '100%',
      display: isActive ? 'flex' : 'none',
      flexDirection: 'column'
    }}>
      {content}
    </Box>
  );
}, (prevProps, nextProps) => {
  // Only re-render if isActive changed or tab identity changed
  return prevProps.isActive === nextProps.isActive && prevProps.tab.id === nextProps.tab.id;
});

TabContentWrapper.displayName = 'TabContentWrapper';

export const OptimizedTabRenderer = React.memo<OptimizedTabRendererProps>(({
  tabs,
  activeTabId
}) => {
  if (tabs.length === 0) {return null;}

  // Keeps all tabs mounted and only shows the active tab (inactive tabs hidden), preserving state
  return (
    <Box sx={{
      width: '100%',
      height: '100%',
      display: 'flex',
      flexDirection: 'column'
    }}>
      {tabs.map(tab => (
        <TabContentWrapper
          key={tab.id}
          tab={tab}
          isActive={tab.id === activeTabId}
        />
      ))}
    </Box>
  );
}, (prevProps, nextProps) => {
  // Only re-render if activeTabId changed or tabs array changed in meaningful ways
  if (prevProps.activeTabId !== nextProps.activeTabId) {
    return false;
  }

  // Check if tabs arrays are the same length
  if (prevProps.tabs.length !== nextProps.tabs.length) {
    return false;
  }

  // Check if tab ids are the same in the same order
  for (let i = 0; i < prevProps.tabs.length; i++) {
    const prevTab = prevProps.tabs[i];
    const nextTab = nextProps.tabs[i];
    if (prevTab!.id !== nextTab!.id) {
      return false;
    }
  }

  return true;
});

OptimizedTabRenderer.displayName = 'OptimizedTabRenderer';

export default OptimizedTabRenderer;