import React, { memo, useMemo, useCallback } from 'react';
import {
  Box,
  Typography,
  List,
  ListItem,
  ListItemButton,
  ListItemText,
  ListItemIcon,
  Divider,
  IconButton,
} from '@mui/material';
import {
  Star as StarIcon,
  Folder as FolderIcon,
  CheckBox as CheckBoxIcon,
  CheckBoxOutlineBlank as CheckBoxOutlineBlankIcon,
} from '@mui/icons-material';
import type { DataSequence } from '@/core/types/dataLibrary';

// Static styles to prevent recreation
const CONTAINER_STYLES = {
  width: 250,
  minWidth: 250,
  flexShrink: 0,
  borderRight: 1,
  borderColor: 'divider',
  overflow: 'auto'
} as const;

const CHECKBOX_STYLES = {
  mr: 1
} as const;

interface SequenceListProps {
  sequences: DataSequence[];
  selectedSequence: DataSequence | null;
  selectedIds: Set<string>;
  onSequenceSelect: (sequence: DataSequence) => void;
  onToggleSelection: (id: string) => void;
}

// Memoized sequence item component for better performance
const SequenceItem = memo<{
  sequence: DataSequence;
  isPinned: boolean;
  isSelected: boolean;
  isChecked: boolean;
  onSelect: (sequence: DataSequence) => void;
  onToggle: (id: string) => void;
}>(({ sequence, isPinned, isSelected, isChecked, onSelect, onToggle }) => {
  const handleSelect = useCallback(() => onSelect(sequence), [onSelect, sequence]);
  const handleToggle = useCallback((e: React.MouseEvent) => {
    e.stopPropagation();
    onToggle(sequence.id);
  }, [onToggle, sequence.id]);

  return (
    <ListItemButton
      selected={isSelected}
      onClick={handleSelect}
    >
      <IconButton
        size="small"
        edge="start"
        onClick={handleToggle}
        role="checkbox"
        aria-label={isChecked ? "Deselect sequence" : "Select sequence"}
        aria-checked={isChecked}
        sx={CHECKBOX_STYLES}
      >
        {isChecked ? (
          <CheckBoxIcon fontSize="small" color="primary" />
        ) : (
          <CheckBoxOutlineBlankIcon fontSize="small" />
        )}
      </IconButton>
      <ListItemIcon>
        {isPinned ? (
          <StarIcon fontSize="small" color="primary" />
        ) : (
          <FolderIcon fontSize="small" />
        )}
      </ListItemIcon>
      <ListItemText 
        primary={sequence.name} 
        secondary={`${sequence.data.length} points`} 
      />
    </ListItemButton>
  );
});

SequenceItem.displayName = 'SequenceItem';

const SequenceList: React.FC<SequenceListProps> = memo(({
  sequences,
  selectedSequence,
  selectedIds,
  onSequenceSelect,
  onToggleSelection,
}) => {
  // Memoized sequence grouping
  const { pinnedSequences, unpinnedSequences } = useMemo(() => ({
    pinnedSequences: sequences.filter(s => s.is_pinned),
    unpinnedSequences: sequences.filter(s => !s.is_pinned)
  }), [sequences]);


  return (
    <Box sx={CONTAINER_STYLES}>
      <List dense>
        <ListItem>
          <Typography variant="subtitle2" color="text.secondary">
            ALL SEQUENCES ({sequences.length})
          </Typography>
        </ListItem>
        
        {pinnedSequences.length > 0 && (
          <>
            <Divider />
            <ListItem>
              <Typography variant="caption" color="text.secondary">PINNED</Typography>
            </ListItem>
            {pinnedSequences.map(seq => (
              <SequenceItem
                key={seq.id}
                sequence={seq}
                isPinned={true}
                isSelected={selectedSequence?.id === seq.id}
                isChecked={selectedIds.has(seq.id)}
                onSelect={onSequenceSelect}
                onToggle={onToggleSelection}
              />
            ))}
          </>
        )}
        
        {unpinnedSequences.length > 0 && (
          <>
            <Divider />
            <ListItem>
              <Typography variant="caption" color="text.secondary">OTHER</Typography>
            </ListItem>
            {unpinnedSequences.map(seq => (
              <SequenceItem
                key={seq.id}
                sequence={seq}
                isPinned={false}
                isSelected={selectedSequence?.id === seq.id}
                isChecked={selectedIds.has(seq.id)}
                onSelect={onSequenceSelect}
                onToggle={onToggleSelection}
              />
            ))}
          </>
        )}
      </List>
    </Box>
  );
});

SequenceList.displayName = 'SequenceList';

export default SequenceList;
