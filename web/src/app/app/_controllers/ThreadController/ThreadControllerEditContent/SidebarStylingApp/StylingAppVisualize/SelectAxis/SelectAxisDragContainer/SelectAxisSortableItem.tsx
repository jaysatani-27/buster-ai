import { useSortable } from '@dnd-kit/sortable';
import React, { useMemo } from 'react';
import { CSS } from '@dnd-kit/utilities';
import { SelectAxisDraggableItemProps } from './interfaces';
import { SelectAxisItem } from './SelectAxisItem';

export const SelectAxisSortableItem: React.FC<SelectAxisDraggableItemProps> = ({
  item,
  zoneId,
  isPlaceholder
}) => {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: item.id,
    data: {
      type: 'item',
      item,
      zoneId
    }
  });

  const style: React.CSSProperties = useMemo(() => {
    return {
      transform: CSS.Transform.toString(transform),
      transition,
      opacity: isDragging ? 0.5 : 1,
      position: 'relative',
      zIndex: isDragging ? 999 : 1
    };
  }, [transform, isDragging]);

  return (
    <SelectAxisItem
      ref={setNodeRef}
      id={item.originalId}
      style={style}
      listeners={listeners}
      attributes={attributes}
      zoneId={zoneId}
      isPlaceholder={isPlaceholder}
    />
  );
};
