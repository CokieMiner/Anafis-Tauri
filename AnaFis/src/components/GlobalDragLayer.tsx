import React, { useState, useEffect, useMemo } from 'react';
import { Box } from '@mui/material';
import DragIndicatorIcon from '@mui/icons-material/DragIndicator';
import type { Tab } from '../types/tabs';
import { bus } from '../utils/ipc';

export const GlobalDragLayer: React.FC = () => {
  const [draggedTab, setDraggedTab] = useState<Tab | null>(null);
  const [clientOffset, setClientOffset] = useState<{ x: number; y: number } | null>(null);

  useEffect(() => {
    const handleDragStart = (tab: Tab) => {
      setDraggedTab(tab);
    };

    const handleMouseMove = (event: MouseEvent) => {
      if (draggedTab) {
        setClientOffset({ x: event.clientX, y: event.clientY });
      }
    };

    const handleDragEnd = () => {
      setDraggedTab(null);
      setClientOffset(null);
    };

    // Listen for drag start events
    bus.on('tab-drag-start', handleDragStart);

    // Listen for mouse movements during drag
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleDragEnd);

    return () => {
      bus.off('tab-drag-start', handleDragStart);
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleDragEnd);
    };
  }, [draggedTab]);

  // Memoize color calculation to match normal tabs
  const colors = useMemo(() => {
    if (!draggedTab) return { primary: '#9c27b0', secondary: '#7b1fa2' };
    if (draggedTab.id === 'home') return { primary: '#9c27b0', secondary: '#7b1fa2' };
    if (draggedTab.id.includes('spreadsheet')) return { primary: '#2196f3', secondary: '#1976d2' };
    if (draggedTab.id.includes('fitting')) return { primary: '#ff9800', secondary: '#f57c00' };
    if (draggedTab.id.includes('solver')) return { primary: '#4caf50', secondary: '#388e3c' };
    if (draggedTab.id.includes('montecarlo')) return { primary: '#e91e63', secondary: '#c2185b' };
    return { primary: '#9c27b0', secondary: '#7b1fa2' };
  }, [draggedTab]);

  if (!draggedTab || !clientOffset) return null;

  return (
    <Box
      sx={{
        position: 'fixed',
        left: clientOffset.x - 20, // Position so mouse stays on drag handle (left side of tab)
        top: clientOffset.y - 18,
        zIndex: 9999,
        pointerEvents: 'none',
        transform: 'rotate(3deg)',
      }}
    >
      <Box
        sx={{
          display: 'flex',
          alignItems: 'center',
          width: '200px', // Match normal tab width
          height: '36px',
          margin: '6px 0',
          padding: '6px 12px',
          borderRadius: '6px',
          backgroundColor: `${colors.primary}E0`, // Match active tab background
          border: `2px solid ${colors.primary}80`, // Match active tab border
          color: colors.primary, // Match active tab text color
          fontSize: '0.8rem',
          fontWeight: 600,
          whiteSpace: 'nowrap',
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          boxShadow: `0 2px 8px ${colors.primary}20`, // Match hover shadow
          userSelect: 'none',
          '&::after': {
            content: '""',
            position: 'absolute',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: 'transparent',
            borderRadius: '5px',
            pointerEvents: 'none',
          },
        }}
      >
        {!draggedTab.id.includes('home') && (
          <Box
            sx={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              mr: 1,
              cursor: 'grab',
              color: 'rgba(255, 255, 255, 0.6)',
              '&:hover': {
                color: '#ffffff',
              },
            }}
          >
            <DragIndicatorIcon sx={{ fontSize: '1rem' }} />
          </Box>
        )}
        {draggedTab.title}
      </Box>
    </Box>
  );
};
