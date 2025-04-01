import React, { useMemo } from 'react';
import type { DraggedItem, DropZoneInternal } from './interfaces';
import { useDroppable } from '@dnd-kit/core';
import { SortableContext, verticalListSortingStrategy } from '@dnd-kit/sortable';
import { SelectAxisSortableItem } from './SelectAxisSortableItem';
import { createStyles } from 'antd-style';
import { green } from 'tailwindcss/colors';
import { StylingLabel } from '../../../Common';
import { SelectAxisSettingsButton } from '../SelectAxisSettingsContent';

export const SelectAxisDropZone: React.FC<{
  zone: DropZoneInternal;
  isError: boolean;
  isOverZone: boolean;
  activeZone: string | null;
  draggedItem: DraggedItem | null;
}> = React.memo(({ zone, isError, isOverZone, activeZone, draggedItem }) => {
  const { styles, cx } = useStyles();
  const { setNodeRef, isOver } = useDroppable({
    id: zone.id,
    data: {
      type: 'zone'
    }
  });

  const showHoverState = (isOver || isOverZone) && !isError;
  const isSameZoneDrag = (isOver || isOverZone) && activeZone === zone.id;

  const extraClass = useMemo(() => {
    if (isError) return 'showError';
    if (isSameZoneDrag) return 'showInternalDrag';
    if (showHoverState) return 'showAddHover';
    return '';
  }, [isError, isSameZoneDrag, showHoverState]);

  const hasItems = zone.items.length > 0;

  return (
    <div ref={setNodeRef}>
      <StylingLabel
        className="!space-y-2"
        label={zone.title}
        labelExtra={<SelectAxisSettingsButton zoneId={zone.id} />}>
        {hasItems && (
          <SortableContext
            items={zone.items.map((item) => item.id)}
            strategy={verticalListSortingStrategy}>
            <div className={cx(styles.container, 'space-y-0.5 transition', extraClass)}>
              {zone.items.map((item) => (
                <SelectAxisSortableItem
                  key={item.originalId}
                  item={item}
                  zoneId={zone.id}
                  isPlaceholder={item.id === draggedItem?.id && draggedItem?.sourceZone !== zone.id}
                />
              ))}
            </div>
          </SortableContext>
        )}

        {!hasItems && <EmptyDropZone className={extraClass} />}
      </StylingLabel>
    </div>
  );
});
SelectAxisDropZone.displayName = 'SelectAxisDropZone';

const EmptyDropZone: React.FC<{
  className: string;
}> = React.memo(({ className }) => {
  const { styles, cx } = useStyles();

  return (
    <div
      className={cx(
        'flex h-[32px] w-full items-center justify-center rounded',
        styles.container,
        className ? className : 'empty'
      )}>
      <span className="dropzone-text select-none">Drag column here</span>
    </div>
  );
});
EmptyDropZone.displayName = 'EmptyDropZone';

const useStyles = createStyles(({ css, token }) => ({
  container: css`
    border-radius: ${token.borderRadius}px;
    transition: all 0.13s ease-in-out;

    &.showError {
      box-shadow: 0 0 3px 1px ${token.colorError};
      background: ${token.colorErrorBg};
      color: ${token.colorError};
    }

    &.showInternalDrag {
      box-shadow: 0 0 3px 1px ${token.colorPrimary};
      color: ${token.colorText};
    }

    &.showAddHover {
      box-shadow: 0 0 3px 1px ${green[500]};
      background: ${green[50]};
      color: ${green[500]};
    }

    &.empty {
      background: transparent;
      border: 0.5px dashed ${token.colorBorder};
    }

    .dropzone-text {
      font-size: ${11}px;
    }

    &.empty span {
      color: ${token.colorTextPlaceholder};
    }
  `
}));
