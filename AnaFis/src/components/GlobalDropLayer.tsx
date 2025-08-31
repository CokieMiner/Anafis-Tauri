import React from 'react';
import { useDrop, useDragLayer } from 'react-dnd';
import { TAB } from '../dndTypes';
import type { Tab } from '../types/tabs';

interface DragItem {
  tab: Tab;
  index: number;
}

interface GlobalDropLayerProps {
  children: React.ReactNode;
  onDetachTab: (tab: Tab) => void;
}

// Drag preview component that follows the mouse
const DragPreview: React.FC = () => {
  const { item, itemType, isDragging, clientOffset } = useDragLayer((monitor) => ({
    item: monitor.getItem(),
    itemType: monitor.getItemType(),
    isDragging: monitor.isDragging(),
    clientOffset: monitor.getClientOffset(),
  }));

  if (!isDragging || itemType !== TAB || !clientOffset) {
    return null;
  }

  const draggedTab = (item as DragItem).tab;

  return (
    <div
      style={{
        position: 'fixed',
        left: clientOffset.x - 70, // Center the preview on mouse
        top: clientOffset.y - 18,
        pointerEvents: 'none',
        zIndex: 9999,
        opacity: 0.8,
      }}
    >
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          minWidth: '140px',
          maxWidth: '180px',
          height: '36px',
          padding: '6px 12px',
          borderRadius: '6px',
          backgroundColor: '#2196f3E0',
          border: '2px solid #2196f380',
          color: '#ffffff',
          fontSize: '0.8rem',
          fontWeight: 600,
          whiteSpace: 'nowrap',
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          boxShadow: '0 8px 25px rgba(33, 150, 243, 0.6)',
          transform: 'rotate(2deg) scale(1.05)',
        }}
      >
        <div
          style={{
            width: '20px',
            height: '20px',
            marginRight: '6px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }}
        >
          <div
            style={{
              width: '14px',
              height: '14px',
              backgroundColor: 'rgba(255, 255, 255, 0.4)',
              borderRadius: '2px',
            }}
          />
        </div>
        {draggedTab.title}
      </div>
    </div>
  );
};

export const GlobalDropLayer: React.FC<GlobalDropLayerProps> = ({ children, onDetachTab }) => {
  const [, drop] = useDrop({
    accept: TAB,
    drop: (item: DragItem) => {
      // Handle tab detachment
      console.log('Tab dropped outside bar, detaching:', item.tab.id);
      onDetachTab(item.tab);
    },
  });

  const attachRef = (element: HTMLDivElement | null) => {
    drop(element);
  };

  return (
    <div
      ref={attachRef}
      style={{
        height: '100vh',
        width: '100vw',
        position: 'relative'
      }}
    >
      {children}
      <DragPreview />
    </div>
  );
};
