import React, { useMemo, useState, useRef } from 'react';
import { SortableContext, horizontalListSortingStrategy} from '@dnd-kit/sortable';
import { useSortable} from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { useTabStore } from '../hooks/useTabStore';
import { Box, TextField, IconButton } from '@mui/material';
import { CloseIcon, HomeIcon, TableChartIcon, TrendingUpIcon, CalculateIcon, CasinoIcon } from '../icons';
import type { Tab } from '../types/tabs';

// Enhanced DraggableTab component with advanced features
function DraggableTab({ tab, isActive, onActivate, onClose }: {
  tab: Tab;
  isActive: boolean;
  onActivate: () => void;
  onClose: () => void;
}) {
  const { renameTab } = useTabStore();
  const isHomeTab = tab.id === 'home';
  const tabRef = useRef<HTMLDivElement>(null);

  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({
    id: tab.id,
    disabled: false, // Remove all drag restrictions for testing
  });

  // State for inline editing
  const [isEditing, setIsEditing] = useState(false);
  const [editingTitle, setEditingTitle] = useState(tab.title);

  // Handle double-click to enter edit mode
  const handleDoubleClick = () => {
    if (!isHomeTab && !isEditing) {
      setIsEditing(true);
      setEditingTitle(tab.title);
    }
  };

  // Handle save (Enter key)
  const handleSave = () => {
    const trimmedTitle = editingTitle.trim();
    if (trimmedTitle && trimmedTitle !== tab.title) {
      renameTab(tab.id, trimmedTitle);
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
  };

  // Get tab icon based on type
  const getTabIcon = () => {
    if (tab.id === 'home') return <HomeIcon sx={{ fontSize: '1rem', color: colors.icon }} />;
    if (tab.id.includes('spreadsheet')) return <TableChartIcon sx={{ fontSize: '1rem', color: colors.icon }} />;
    if (tab.id.includes('fitting')) return <TrendingUpIcon sx={{ fontSize: '1rem', color: colors.icon }} />;
    if (tab.id.includes('solver')) return <CalculateIcon sx={{ fontSize: '1rem', color: colors.icon }} />;
    if (tab.id.includes('montecarlo')) return <CasinoIcon sx={{ fontSize: '1rem', color: colors.icon }} />;
    return <HomeIcon sx={{ fontSize: '1rem', color: colors.icon }} />;
  };

  // Enhanced style calculation with tilt for phantom
  const style = useMemo(() => ({
    transform: isDragging
      ? `${CSS.Transform.toString(transform)} scale(0.9) rotate(3deg)`
      : CSS.Transform.toString(transform),
    transition: isDragging ? 'none' : transition,
    opacity: isDragging ? 0.3 : 1,
    zIndex: isDragging ? 1000 : 1,
  }), [transform, transition, isDragging]);

  // Color calculation based on tab type
  const colors = useMemo(() => {
    if (tab.id === 'home') return { primary: '#9c27b0', secondary: '#7b1fa2', accent: '#ba68c8', icon: '#ba68c8' };
    if (tab.id.includes('spreadsheet')) return { primary: '#2196f3', secondary: '#1976d2', accent: '#64b5f6', icon: '#64b5f6' };
    if (tab.id.includes('fitting')) return { primary: '#ff9800', secondary: '#f57c00', accent: '#ffb74d', icon: '#ffb74d' };
    if (tab.id.includes('solver')) return { primary: '#4caf50', secondary: '#388e3c', accent: '#81c784', icon: '#81c784' };
    if (tab.id.includes('montecarlo')) return { primary: '#e91e63', secondary: '#c2185b', accent: '#f06292', icon: '#f06292' };
    return { primary: '#9c27b0', secondary: '#7b1fa2', accent: '#ba68c8', icon: '#ba68c8' };
  }, [tab.id]);

  const attachDragRef = (element: HTMLDivElement | null) => {
    setNodeRef(element);
    if (tabRef.current !== element) {
      tabRef.current = element;
    }
  };

  return (
    <Box
      ref={attachDragRef}
      style={style}
      {...attributes}
      onClick={(e) => {
        // Only activate if not clicking on drag handle or close button
        if (!e.defaultPrevented && !isEditing) {
          onActivate();
        }
      }}
      sx={{
        position: 'relative',
        display: 'flex',
        alignItems: 'center',
        minWidth: '200px',
        maxWidth: '200px',
        height: '44px',
        margin: '4px 2px',
        padding: '8px 12px',
        borderRadius: '8px',
        cursor: 'default',
        userSelect: 'none',
        transition: 'all 0.2s ease-in-out',
        background: isActive
          ? colors.primary
          : isDragging
            ? 'rgba(255, 255, 255, 0.03)'
            : 'rgba(255, 255, 255, 0.02)',
        border: `2px solid ${isActive ? colors.accent : 'rgba(255, 255, 255, 0.08)'}`,
        color: isActive || isDragging
          ? '#ffffff'
          : 'rgba(255, 255, 255, 0.7)',
        boxShadow: isActive
          ? `0 2px 8px ${colors.primary}40`
          : isDragging
            ? `0 2px 8px rgba(255, 255, 255, 0.1)`
            : 'none',
        '&:hover': {
          background: isActive
            ? colors.primary
            : isDragging
              ? colors.secondary
              : 'rgba(255, 255, 255, 0.08)',
          borderColor: isActive
            ? colors.accent
            : colors.primary,
          color: '#ffffff',
          transform: isDragging ? 'none' : 'translateY(-1px)',
          boxShadow: isActive
            ? `0 4px 12px ${colors.primary}50`
            : `0 2px 8px ${colors.primary}20`,
        },
        '&::before': isDragging ? {
          content: '""',
          position: 'absolute',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          borderRadius: '8px',
          background: `rgba(255, 255, 255, 0.1)`,
          zIndex: -1,
        } : {},
      }}
    >
      {/* Enhanced Drag Handle */}
      {!isHomeTab && (
        <Box
          {...listeners}
          onClick={(e) => {
            e.preventDefault();
            e.stopPropagation();
          }}
          sx={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            width: '24px',
            height: '28px',
            marginRight: '6px',
            cursor: isDragging ? 'grabbing' : 'grab',
            transition: 'all 0.15s ease-out',
            position: 'relative',
            flexShrink: 0,
          }}
          title="Drag to reorder or detach tab"
        >
          {/* Drag indicator - three fat horizontal lines */}
          <Box
            sx={{
              display: 'flex',
              flexDirection: 'column',
              gap: '3px',
              alignItems: 'center',
              justifyContent: 'center',
              width: '100%',
              height: '100%',
            }}
          >
            <Box sx={{
              width: '18px',
              height: '3px',
              backgroundColor: `${colors.accent} !important`,
              background: `${colors.accent} !important`,
              color: `${colors.accent} !important`,
              borderRadius: '2px',
              boxShadow: '0 1px 2px rgba(0, 0, 0, 0.5)',
              border: 'none',
            }} />
            <Box sx={{
              width: '18px',
              height: '3px',
              backgroundColor: `${colors.accent} !important`,
              background: `${colors.accent} !important`,
              color: `${colors.accent} !important`,
              borderRadius: '2px',
              boxShadow: '0 1px 2px rgba(0, 0, 0, 0.5)',
              border: 'none',
            }} />
            <Box sx={{
              width: '18px',
              height: '3px',
              backgroundColor: `${colors.accent} !important`,
              background: `${colors.accent} !important`,
              color: `${colors.accent} !important`,
              borderRadius: '2px',
              boxShadow: '0 1px 2px rgba(0, 0, 0, 0.5)',
              border: 'none',
            }} />
          </Box>
        </Box>
      )}

      {/* Tab Icon */}
      <Box sx={{
        mr: 1,
        display: 'flex',
        alignItems: 'center',
        flexShrink: 0
      }}>
        {getTabIcon()}
      </Box>

      {/* Editable Title */}
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
              fontSize: '0.85rem',
              fontWeight: isActive ? 600 : 500,
              color: '#ffffff',
              '&:before': { borderBottomColor: 'transparent' },
              '&:hover:before': { borderBottomColor: colors.accent },
              '&:after': { borderBottomColor: colors.accent },
              '& .MuiInput-input': {
                padding: '2px 0',
              },
            },
          }}
        />
      ) : (
        <Box
          onDoubleClick={handleDoubleClick}
          sx={{
            flex: 1,
            fontWeight: isActive ? 600 : 500,
            fontSize: '0.85rem',
            whiteSpace: 'nowrap',
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            mr: 1,
            letterSpacing: '0.025em',
            cursor: isHomeTab ? 'default' : 'text',
            display: 'flex',
            alignItems: 'center',
            minWidth: 0, // Allow text to shrink
          }}
          title={tab.title}
        >
          {tab.title}
        </Box>
      )}

      {/* Close Button */}
      {!isHomeTab && (
        <IconButton
          onClick={(e) => {
            e.stopPropagation();
            onClose();
          }}
          size="small"
          sx={{
            width: '20px',
            height: '20px',
            padding: '2px',
            marginLeft: '4px',
            color: 'rgba(255, 255, 255, 0.5)',
            borderRadius: '3px',
            transition: 'all 0.15s ease-out',
            '&:hover': {
              backgroundColor: 'rgba(244, 67, 54, 0.15)',
              color: '#ff6b6b',
            },
            '&:active': {
              transform: 'scale(0.95)',
            },
          }}
        >
          <CloseIcon sx={{ fontSize: '0.9rem' }} />
        </IconButton>
      )}
    </Box>
  );
}

// Memoize the component to prevent unnecessary re-renders
const MemoizedDraggableTab = React.memo(DraggableTab);

export function DraggableTabBar() {
  const { tabs, activeTabId, setActiveTab, removeTab } = useTabStore();
  const tabBarRef = useRef<HTMLDivElement>(null);

  // Ensure home tab is always first, then sort other tabs
  const sortedTabs = useMemo(() => {
    const homeTab = tabs.find(tab => tab.id === 'home');
    const otherTabs = tabs.filter(tab => tab.id !== 'home');
    const result = homeTab ? [homeTab, ...otherTabs] : tabs;
    return result;
  }, [tabs]);

  // Memoize tab IDs for SortableContext (excluding home tab from sorting)
  const sortableTabIds = useMemo(() => {
    const ids = sortedTabs.filter(tab => tab.id !== 'home').map((t: Tab) => t.id);
    return ids;
  }, [sortedTabs]);

  return (
    <Box
      ref={tabBarRef}
      data-testid="tab-bar"
      sx={{
        borderBottom: '1px solid rgba(255, 255, 255, 0.08)',
        backgroundColor: 'rgba(18, 18, 18, 0.95)',
        position: 'relative',
        width: '100%',
      }}
    >
        <Box sx={{
          display: 'flex',
          alignItems: 'center',
          minHeight: '52px',
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
          {/* Render home tab first (not draggable) */}
          {sortedTabs.filter(tab => tab.id === 'home').map((tab: Tab) => (
            <Box
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              sx={{
                position: 'relative',
                display: 'flex',
                alignItems: 'center',
                minWidth: '160px',
                maxWidth: '200px',
                height: '44px',
                margin: '4px 2px',
                padding: '8px 12px',
                borderRadius: '8px',
                cursor: 'pointer',
                userSelect: 'none',
                transition: 'all 0.2s ease-in-out',
                background: activeTabId === tab.id
                  ? '#9c27b0'
                  : 'rgba(255, 255, 255, 0.05)',
                border: activeTabId === tab.id
                  ? `2px solid #ba68c8`
                  : '1px solid rgba(255, 255, 255, 0.12)',
                color: activeTabId === tab.id
                  ? '#ffffff'
                  : 'rgba(255, 255, 255, 0.7)',
                boxShadow: activeTabId === tab.id
                  ? `0 2px 8px rgba(156, 39, 176, 0.4)`
                  : 'none',
                '&:hover': {
                  background: activeTabId === tab.id
                    ? '#9c27b0'
                    : `rgba(255, 255, 255, 0.08)`,
                  borderColor: activeTabId === tab.id
                    ? '#ba68c8'
                    : '#9c27b0',
                  color: '#ffffff',
                  transform: 'translateY(-1px)',
                  boxShadow: activeTabId === tab.id
                    ? `0 4px 12px rgba(156, 39, 176, 0.5)`
                    : `0 2px 8px rgba(156, 39, 176, 0.2)`,
                },
              }}
            >
              {/* Home Icon */}
              <Box sx={{ mr: 1, display: 'flex', alignItems: 'center', flexShrink: 0 }}>
                <HomeIcon sx={{ fontSize: '1rem', color: '#ba68c8' }} />
              </Box>

              {/* Title */}
              <Box sx={{
                flex: 1,
                fontWeight: activeTabId === tab.id ? 600 : 500,
                fontSize: '0.85rem',
                whiteSpace: 'nowrap',
                overflow: 'hidden',
                textOverflow: 'ellipsis',
                mr: 1,
                letterSpacing: '0.025em',
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                minWidth: 0,
              }}>
                {tab.title}
              </Box>
            </Box>
          ))}

          {/* Render other tabs in sortable context */}
          <SortableContext items={sortableTabIds} strategy={horizontalListSortingStrategy}>
            {sortedTabs.filter(tab => tab.id !== 'home').map((tab: Tab) => {
              return (
                <MemoizedDraggableTab
                  key={tab.id}
                  tab={tab}
                  isActive={activeTabId === tab.id}
                  onActivate={() => setActiveTab(tab.id)}
                  onClose={async () => {
                    await removeTab(tab.id);
                  }}
                />
              );
          })}
        </SortableContext>
      </Box>
    </Box>
  );
}