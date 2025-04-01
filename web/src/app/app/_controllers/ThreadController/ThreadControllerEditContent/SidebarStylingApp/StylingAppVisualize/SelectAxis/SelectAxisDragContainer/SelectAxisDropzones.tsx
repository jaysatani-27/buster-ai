import React, { useMemo } from 'react';
import {
  DndContext,
  useSensor,
  useSensors,
  MouseSensor,
  TouchSensor,
  pointerWithin
} from '@dnd-kit/core';
import { SelectAxisDropZone } from './SelectAxisDropzone';
import { AvailableItemsList } from './SelectAxisAvailableItemsList';
import { SelectAxisDragOverlay } from './SelectAxisDraggingItem';
import { useDropzonesInternal } from './useDropzonesInternal';
import { DropZone, SelectAxisItem } from './interfaces';
import { useDropzonesExternal } from './useDropzonesExternal';
import { SelectAxisContainerId } from '../config';

export const SelectAxisDropzones: React.FC<{
  items: SelectAxisItem[];
  dropZones: DropZone[];
  onChange: (dropZones: DropZone[]) => void;
}> = ({ items = [], dropZones: dropZonesExternal = [], onChange }) => {
  const { onDropzonesChange } = useDropzonesExternal({ onChange });

  const {
    draggedItem,
    handleDragEnd,
    overZoneId,
    handleDragOver,
    handleDragStart,
    dropZones,
    errorZone,
    activeZone,
    availableItems
  } = useDropzonesInternal({ items, dropZonesExternal, onDropzonesChange });

  const draggedItemOriginalId: string | null = useMemo(() => {
    if (!draggedItem) return null;
    const item = availableItems.find(({ originalId }) => originalId === draggedItem.originalId);
    if (!item) return null;
    return item.originalId;
  }, [draggedItem?.originalId, availableItems]);

  const sensors = useSensors(
    useSensor(MouseSensor, { activationConstraint: { distance: 1 } }),
    useSensor(TouchSensor, { activationConstraint: { distance: 1 } })
  );

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={pointerWithin}
      onDragStart={handleDragStart}
      onDragOver={handleDragOver}
      onDragEnd={handleDragEnd}>
      <div className="flex flex-col gap-4">
        {dropZones.map((zone) => (
          <SelectAxisDropZone
            key={zone.id}
            zone={zone}
            isError={errorZone?.zoneId === zone.id}
            isOverZone={overZoneId === zone.id}
            activeZone={activeZone}
            draggedItem={draggedItem}
          />
        ))}

        {availableItems.length > 0 && (
          <AvailableItemsList
            items={availableItems}
            activeZone={activeZone}
            isActive={overZoneId === SelectAxisContainerId.Available}
          />
        )}
      </div>

      <SelectAxisDragOverlay
        draggedItem={draggedItem}
        draggedItemOriginalId={draggedItemOriginalId}
      />
    </DndContext>
  );
};
