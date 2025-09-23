import React, { useEffect } from 'react';
import CustomTitleBar from './CustomTitleBar';
import { useTabStore } from '../hooks/useTabStore';
import { getTabContent } from '../utils/tabs';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import Box from '@mui/material/Box';

export const DetachedTabWindow = () => {
  const { tabs, addTab: storeAddTab } = useTabStore();

  const handleReattachTab = async () => {
    try {
      const urlParams = new URLSearchParams(window.location.search);
      const tabId = urlParams.get('tabId');
      const tabType = urlParams.get('tabType');
      const tabTitle = urlParams.get('tabTitle');

      if (tabId && tabType && tabTitle) {
        const currentWindow = getCurrentWindow();
        await invoke('send_tab_to_main', {
          tab_info: {
            id: tabId,
            title: decodeURIComponent(tabTitle),
            content_type: tabType,
            state: {},
            icon: null
          },
          window_id: currentWindow.label,
        });
        await currentWindow.close();
      }
    } catch (error) {
      console.error('Failed to reattach tab', error);
    }
  };

  useEffect(() => {
    if (tabs.length === 0) {
      (async () => {
        try {
          const urlParams = new URLSearchParams(window.location.search);
          const tabId = urlParams.get('tabId');
          const tabType = urlParams.get('tabType');
          const tabTitle = urlParams.get('tabTitle');

          if (tabId && tabType && tabTitle) {
            const tabContent = getTabContent(tabType, () => {});
            storeAddTab({ id: tabId, title: decodeURIComponent(tabTitle), content: tabContent });
          }
        } catch (error) {
          console.error("Failed to initialize detached tab", error);
        }
      })();
    }
  }, [storeAddTab, tabs.length]);

  const activeTab = tabs[0];

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
      <CustomTitleBar
        title={activeTab?.title || 'AnaFis'}
        isDetachedTabWindow={true}
        onReattach={handleReattachTab}
      />
      <Box sx={{
        flexGrow: 1,
        p: 0,
        bgcolor: '#0a0a0a',
        overflow: 'auto',
        width: '100%',
        margin: 0
      }}>
        {activeTab?.content}
      </Box>
    </Box>
  );
};
