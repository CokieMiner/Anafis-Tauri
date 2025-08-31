import React, { useMemo, useState, useEffect } from 'react';
import {
  SortableContext,
  horizontalListSortingStrategy,
} from '@dnd-kit/sortable';
import {
  useSortable,
} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { useTabStore } from '../hooks/useTabStore';
import { bus } from '../utils/ipc';
import { Box, IconButton, Typography, TextField } from '@mui/material';
import CloseIcon from '@mui/icons-material/Close';
import DragIndicatorIcon from '@mui/icons-material/DragIndicator';
import type { Tab } from '../types/tabs';

interface DraggableTabProps {
  tab: Tab;
  isActive: boolean;
  onActivate: () => void;
  onClose: () => void;
  onRename: (newTitle: string) => void;
}

function DraggableTab({ tab, isActive, onActivate, onClose, onRename }: DraggableTabProps) {
  const isHomeTab = tab.id === 'home';

  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({
    id: tab.id,
    disabled: isHomeTab, // Disable dragging for home tab
    data: {
      tab: tab, // Pass the tab data for the drag layer
    },
  });

  // State for inline editing
  const [isEditing, setIsEditing] = useState(false);
  const [editingTitle, setEditingTitle] = useState(tab.title);

  // Handle double-click to enter edit mode
  const handleDoubleClick = (e: React.MouseEvent) => {
    e.preventDefault(); // Prevent default double-click behavior
    if (!isHomeTab && !isEditing) {
      setIsEditing(true);
      setEditingTitle(tab.title);
    }
  };

  // Handle save (Enter key)
  const handleSave = () => {
    if (editingTitle.trim() && editingTitle.trim() !== tab.title) {
      onRename(editingTitle.trim());
    }
    setIsEditing(false);
  };

  // Handle cancel (Escape key)
  const handleCancel = () => {
    setIsEditing(false);
    setEditingTitle(tab.title);
  };

  // Handle key press in input
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleSave();
    } else if (e.key === 'Escape') {
      handleCancel();
    }
  };

  // Handle input blur (save on blur)
  const handleBlur = () => {
    handleSave();
  };  // Memoize the style calculation
  const style = useMemo(() => ({
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.3 : 1, // More transparent when dragging
  }), [transform, transition, isDragging]);

  // Memoize color calculation
  const colors = useMemo(() => {
    if (tab.id === 'home') return { primary: '#9c27b0', secondary: '#7b1fa2' };
    if (tab.id.includes('spreadsheet')) return { primary: '#2196f3', secondary: '#1976d2' };
    if (tab.id.includes('fitting')) return { primary: '#ff9800', secondary: '#f57c00' };
    if (tab.id.includes('solver')) return { primary: '#4caf50', secondary: '#388e3c' };
    if (tab.id.includes('montecarlo')) return { primary: '#e91e63', secondary: '#c2185b' };
    return { primary: '#9c27b0', secondary: '#7b1fa2' };
  }, [tab.id]);

  const attachDragRef = (element: HTMLDivElement | null) => {
    setNodeRef(element);
  };

  return (
    <Box
      ref={attachDragRef}
      style={style}
      {...attributes}
      onClick={() => {
        console.log('Tab clicked for activation:', tab.id);
        onActivate();
      }}
      sx={{
        position: 'relative',
        display: 'flex',
        alignItems: 'center',
        width: '200px', // Fixed width to prevent expansion
        height: '36px',
        margin: '6px 0',
        padding: '6px 12px',
        borderRadius: '6px',
        cursor: isHomeTab ? 'default' : 'pointer',
        userSelect: 'none',
        transition: 'all 0.1s ease-out', // Even faster transition
        backgroundColor: isActive
          ? `${colors.primary}E0`
          : 'rgba(255, 255, 255, 0.02)',
        border: isActive
          ? `2px solid ${colors.primary}80`
          : '1px solid rgba(255, 255, 255, 0.15)',
        color: isActive
          ? colors.primary
          : 'rgba(255, 255, 255, 0.6)',
        zIndex: isDragging ? 1000 : 1,
        '&::after': {
          content: '""',
          position: 'absolute',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          backgroundColor: isActive
            ? 'transparent'
            : `${colors.primary}08`,
          borderRadius: '5px',
          pointerEvents: 'none',
        },
        '&:hover': {
          backgroundColor: isActive
            ? `${colors.primary}F0`
            : `${colors.primary}15`,
          borderColor: isActive
            ? colors.primary
            : `${colors.primary}40`,
          color: isActive ? colors.primary : '#ffffff',
          transform: 'translateY(-1px)',
          boxShadow: `0 2px 8px ${colors.primary}20`,
        },
      }}
    >
      {!isHomeTab && (
        <Box
          {...listeners}
          sx={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            width: '20px',
            height: '20px',
            marginRight: '6px',
            cursor: isDragging ? 'grabbing' : 'grab',
            borderRadius: '3px',
            '&:hover': {
              backgroundColor: 'rgba(255, 255, 255, 0.1)',
            },
          }}
        >
          <DragIndicatorIcon
            sx={{
              fontSize: '14px',
              color: 'rgba(255, 255, 255, 0.4)',
              '&:hover': {
                color: 'rgba(255, 255, 255, 0.6)',
              },
            }}
          />
        </Box>
      )}

      {isEditing ? (
        <TextField
          autoFocus
          value={editingTitle}
          onChange={(e) => setEditingTitle(e.target.value)}
          onKeyDown={handleKeyDown}
          onBlur={handleBlur}
          variant="standard"
          sx={{
            flex: 1,
            mr: 1,
            '& .MuiInput-root': {
              fontSize: '0.8rem',
              fontWeight: isActive ? 600 : 400,
              color: isActive ? colors.primary : 'rgba(255, 255, 255, 0.6)',
              letterSpacing: '0.025em',
            },
            '& .MuiInput-underline:before': {
              borderBottomColor: 'transparent',
            },
            '& .MuiInput-underline:hover:before': {
              borderBottomColor: colors.primary,
            },
            '& .MuiInput-underline:after': {
              borderBottomColor: colors.primary,
            },
          }}
        />
      ) : (
        <Typography
          variant="body2"
          onDoubleClick={handleDoubleClick}
          sx={{
            flex: 1,
            fontWeight: isActive ? 600 : 400,
            fontSize: '0.8rem',
            whiteSpace: 'nowrap',
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            mr: 1,
            letterSpacing: '0.025em',
            cursor: isHomeTab ? 'default' : 'text',
          }}
        >
          {tab.title}
        </Typography>
      )}

      {tab.id !== 'home' && (
        <IconButton
          size="small"
          onClick={(e) => {
            console.log('Close button clicked for tab:', tab.id);
            e.stopPropagation();
            onClose();
          }}
          sx={{
            width: '18px',
            height: '18px',
            padding: '1px',
            marginLeft: '4px',
            color: 'rgba(255, 255, 255, 0.4)',
            borderRadius: '3px',
            transition: 'all 0.1s ease-out', // Faster transition
            '&:hover': {
              color: '#ff6b6b',
              backgroundColor: 'rgba(255, 107, 107, 0.1)',
              transform: 'scale(1.1)',
            },
          }}
        >
          <CloseIcon sx={{ fontSize: '12px' }} />
        </IconButton>
      )}
    </Box>
  );
}

// Memoize the component to prevent unnecessary re-renders
const MemoizedDraggableTab = React.memo(DraggableTab);

export function DraggableTabBar() {
  const { tabs, activeTabId, setActiveTab, removeTab, renameTab } = useTabStore();

  console.log('DraggableTabBar render:', { tabs: tabs.map(t => t.id), activeTabId, tabsLength: tabs.length });
  const tabBarRef = React.useRef<HTMLDivElement>(null);

  // Check if this is a detached tab window
  const [isDetachedTabWindow, setIsDetachedTabWindow] = useState(false);
  useEffect(() => {
    const urlParams = new URLSearchParams(window.location.search);
    const detached = urlParams.get('detached') === 'true';
    const tabId = urlParams.get('tabId');
    const isDetached = detached && !!tabId;
    console.log('DraggableTabBar: checking if detached window:', {
      detached,
      tabId,
      isDetached,
      fullUrl: window.location.search,
      allParams: Object.fromEntries(urlParams.entries())
    });
    setIsDetachedTabWindow(isDetached);
  }, []);

  // Listen for cross-window tab drops
  useEffect(() => {
    const onExternalDrop = (tab: Tab) => {
      // Another window dropped a tab on this tab bar
      console.log('Received external tab drop:', tab);
      // Add the tab to this window
      const newTab: Tab = {
        id: tab.id,
        title: tab.title,
        content: tab.content,
      };
      // Use the store's addTab function
      const { addTab } = useTabStore.getState();
      addTab(newTab);
    };

    bus.on('tab-drop', onExternalDrop);
    return () => {
      bus.off('tab-drop', onExternalDrop);
    };
  }, []);

  // Memoize tab IDs for SortableContext, including home tab but it will be disabled
  const tabIds = useMemo(() => tabs.map((t: Tab) => t.id), [tabs]);

  return (
    <Box
      ref={tabBarRef}
      sx={{
        borderBottom: '1px solid rgba(255, 255, 255, 0.08)',
        backgroundColor: 'rgba(18, 18, 18, 0.95)',
        position: 'relative',
        backdropFilter: 'blur(10px)',
      }}
    >
      <SortableContext items={tabIds} strategy={horizontalListSortingStrategy}>
        <Box sx={{
          display: 'flex',
          alignItems: 'center',
            minHeight: '48px',
            px: 2,
            gap: 1,
            overflow: 'auto',
            scrollbarWidth: 'thin',
            scrollbarColor: 'rgba(255, 255, 255, 0.15) transparent',
            '&::-webkit-scrollbar': {
              height: '3px',
            },
            '&::-webkit-scrollbar-track': {
              background: 'transparent',
            },
            '&::-webkit-scrollbar-thumb': {
              background: 'rgba(255, 255, 255, 0.15)',
              borderRadius: '2px',
            },
          }}>
            {/* Render all tabs in sortable context */}
            {tabs.map((tab: Tab) => {
              console.log('Rendering tab:', { id: tab.id, title: tab.title, isHome: tab.id === 'home' });
              return (
                <MemoizedDraggableTab
                  key={tab.id}
                  tab={tab}
                  isActive={activeTabId === tab.id}
                  onActivate={() => setActiveTab(tab.id)}
                  onClose={async () => {
                    console.log('Close button clicked for tab:', tab.id, 'isDetachedTabWindow:', isDetachedTabWindow, 'tabs length:', tabs.length);
                    console.log('Current tabs:', tabs.map(t => ({ id: t.id, title: t.title })));

                    // If this is a detached tab window with only one tab, close the window
                    if (isDetachedTabWindow && tabs.length === 1) {
                      try {
                        console.log('Closing detached window...');
                        const { getCurrentWindow } = await import('@tauri-apps/api/window');
                        const currentWindow = getCurrentWindow();
                        console.log('Got current window, attempting to close...');
                        await currentWindow.close();
                        console.log('Window close command completed');
                      } catch (error) {
                        console.error('Failed to close window:', error);
                        // Fallback: try to remove the tab normally
                        console.log('Falling back to normal tab removal...');
                        await removeTab(tab.id);
                      }
                    } else {
                      // Normal tab removal
                      console.log('Normal tab removal for tab:', tab.id);
                      await removeTab(tab.id);
                    }
                  }}
                  onRename={(newTitle: string) => {
                    console.log('Renaming tab:', tab.id, 'to:', newTitle);
                    renameTab(tab.id, newTitle);
                  }}
                />
              );
            })}
          </Box>
        </SortableContext>

    </Box>
  );
}
