import React, { useState } from 'react';
import type { Tab } from '../types/tabs';
import Box from '@mui/material/Box';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import CloseIcon from '@mui/icons-material/Close';

interface TabBarProps {
  tabs: Tab[];
  activeTabId: string | null;
  setActiveTabId: (id: string) => void;
  removeTab: (id: string) => void;
  onReorderTabs?: (sourceIndex: number, destinationIndex: number) => void;
}

const TabBar: React.FC<TabBarProps> = ({ tabs, activeTabId, setActiveTabId, removeTab, onReorderTabs }) => {
  const [draggedTab, setDraggedTab] = useState<string | null>(null);
  const [dragOverIndex, setDragOverIndex] = useState<number | null>(null);

  const handleDragStart = (e: React.DragEvent, tabId: string) => {
    setDraggedTab(tabId);
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('text/plain', tabId);
  };

  const handleDragOver = (e: React.DragEvent, index: number) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    setDragOverIndex(index);
  };

  const handleDragLeave = () => {
    setDragOverIndex(null);
  };

  const handleDrop = (e: React.DragEvent, dropIndex: number) => {
    e.preventDefault();
    const draggedTabId = e.dataTransfer.getData('text/plain');
    const draggedIndex = tabs.findIndex(tab => tab.id === draggedTabId);

    if (draggedIndex !== -1 && draggedIndex !== dropIndex && onReorderTabs) {
      onReorderTabs(draggedIndex, dropIndex);
    }

    setDraggedTab(null);
    setDragOverIndex(null);
  };

  const handleDragEnd = () => {
    setDraggedTab(null);
    setDragOverIndex(null);
  };

  // Get vibrant colors for different tab types
  const getTabColors = (tabId: string) => {
    if (tabId === 'home') return { primary: '#9c27b0', secondary: '#7b1fa2' }; // Vibrant Purple for home
    if (tabId.includes('spreadsheet')) return { primary: '#2196f3', secondary: '#1976d2' };
    if (tabId.includes('fitting')) return { primary: '#ff9800', secondary: '#f57c00' };
    if (tabId.includes('solver')) return { primary: '#4caf50', secondary: '#388e3c' };
    if (tabId.includes('montecarlo')) return { primary: '#e91e63', secondary: '#c2185b' };
    return { primary: '#9c27b0', secondary: '#7b1fa2' }; // Default purple
  };

  return (
    <Box sx={{
      borderBottom: '1px solid rgba(255, 255, 255, 0.08)',
      backgroundColor: 'rgba(18, 18, 18, 0.95)',
      position: 'relative',
      backdropFilter: 'blur(10px)',
    }}>
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
        {tabs.map((tab, index) => {
          const colors = getTabColors(tab.id);
          const isDragged = draggedTab === tab.id;
          const isDragOver = dragOverIndex === index;

          return (
            <Box
              key={tab.id}
              draggable
              onDragStart={(e: React.DragEvent<HTMLDivElement>) => handleDragStart(e, tab.id)}
              onDragOver={(e: React.DragEvent<HTMLDivElement>) => handleDragOver(e, index)}
              onDragLeave={handleDragLeave}
              onDrop={(e: React.DragEvent<HTMLDivElement>) => handleDrop(e, index)}
              onDragEnd={handleDragEnd}
              sx={{
                position: 'relative',
                display: 'flex',
                alignItems: 'center',
                minWidth: '140px',
                maxWidth: '180px',
                height: '36px',
                margin: '6px 0',
                padding: '6px 12px',
                borderRadius: '6px',
                cursor: 'pointer',
                userSelect: 'none',
                transition: 'all 0.2s ease-in-out',
                backgroundColor: activeTabId === tab.id
                  ? `${colors.primary}E0`
                  : 'rgba(255, 255, 255, 0.02)',
                border: activeTabId === tab.id
                  ? `2px solid ${colors.primary}80`
                  : `1px solid rgba(255, 255, 255, 0.15)`,
                color: activeTabId === tab.id
                  ? colors.primary
                  : 'rgba(255, 255, 255, 0.6)',
                transform: isDragged ? 'scale(1.02)' : 'scale(1)',
                opacity: isDragged ? 0.8 : 1,
                zIndex: isDragged ? 1000 : activeTabId === tab.id ? 100 : 1,
                '&::before': {
                  content: '""',
                  position: 'absolute',
                  left: isDragOver && dragOverIndex! < index ? '-1px' : 'auto',
                  right: isDragOver && dragOverIndex! > index ? '-1px' : 'auto',
                  top: '50%',
                  transform: 'translateY(-50%)',
                  width: '3px',
                  height: '70%',
                  backgroundColor: colors.primary,
                  borderRadius: '2px',
                  opacity: isDragOver ? 1 : 0,
                  transition: 'opacity 0.2s ease',
                },
                '&::after': {
                  content: '""',
                  position: 'absolute',
                  top: 0,
                  left: 0,
                  right: 0,
                  bottom: 0,
                  backgroundColor: activeTabId === tab.id
                    ? 'transparent'
                    : `${colors.primary}08`,
                  borderRadius: '5px',
                  pointerEvents: 'none',
                },
                '&:hover': {
                  backgroundColor: activeTabId === tab.id
                    ? `${colors.primary}F0`
                    : `${colors.primary}15`,
                  borderColor: activeTabId === tab.id
                    ? colors.primary
                    : `${colors.primary}40`,
                  color: activeTabId === tab.id ? colors.primary : '#ffffff',
                  transform: 'translateY(-1px)',
                  boxShadow: `0 2px 8px ${colors.primary}20`,
                },
              }}
              onClick={() => setActiveTabId(tab.id)}
            >
              <Typography
                variant="body2"
                sx={{
                  flex: 1,
                  fontWeight: activeTabId === tab.id ? 600 : 400,
                  fontSize: '0.8rem',
                  whiteSpace: 'nowrap',
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                  mr: 1,
                  letterSpacing: '0.025em',
                }}
              >
                {tab.title}
              </Typography>

              {tab.id !== 'home' && (
                <IconButton
                  size="small"
                  onClick={(e) => {
                    e.stopPropagation();
                    removeTab(tab.id);
                  }}
                  sx={{
                    width: '18px',
                    height: '18px',
                    padding: '1px',
                    marginLeft: '4px',
                    color: 'rgba(255, 255, 255, 0.4)',
                    borderRadius: '3px',
                    transition: 'all 0.15s ease-in-out',
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
        })}
      </Box>
    </Box>
  );
};

export default TabBar;
