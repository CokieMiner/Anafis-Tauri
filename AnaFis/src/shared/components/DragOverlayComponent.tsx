// Drag overlay component for visual feedback during tab dragging
import React from 'react';
import { Box, useTheme, alpha } from '@mui/material';
import { getTabIcon, getTabColors } from '@/core/utils/tabColors';

interface DragOverlayComponentProps {
  draggedTab: {
    id: string;
    title: string;
    type: string;
  };
}

export const DragOverlayComponent = React.memo<DragOverlayComponentProps>(({ draggedTab }) => {
  const colors = getTabColors(draggedTab.type);
  const theme = useTheme();

  return (
    <Box
      sx={{
        display: 'flex',
        alignItems: 'center',
        minWidth: '160px',
        maxWidth: '200px',
        height: '44px',
        margin: '4px 2px',
        padding: '8px 12px',
        borderRadius: '8px',
        backgroundColor: alpha(theme.palette.background.paper, 0.8),
        border: `2px solid ${colors.accent}`,
        color: theme.palette.text.primary,
        boxShadow: `0 4px 12px ${alpha(colors.primary, 0.3)}`,
        transform: 'rotate(5deg)',
        opacity: 0.9,
        pointerEvents: 'none',
        zIndex: 9999,
      }}
    >
      {/* Tab Icon */}
      <Box sx={{ mr: 1, display: 'flex', alignItems: 'center', flexShrink: 0 }}>
        {getTabIcon(draggedTab.type, '1rem')}
      </Box>

      {/* Title */}
      <Box sx={{
        flex: 1,
        fontWeight: 600,
        fontSize: '0.85rem',
        whiteSpace: 'nowrap',
        overflow: 'hidden',
        textOverflow: 'ellipsis',
        mr: 1,
        letterSpacing: '0.025em',
      }}>
        {draggedTab.title}
      </Box>
    </Box>
  );
});

DragOverlayComponent.displayName = 'DragOverlayComponent';

export default DragOverlayComponent;