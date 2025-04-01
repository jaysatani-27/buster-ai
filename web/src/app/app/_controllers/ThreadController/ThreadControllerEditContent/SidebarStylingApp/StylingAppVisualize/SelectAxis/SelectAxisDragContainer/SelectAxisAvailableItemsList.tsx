import React from 'react';
import { SelectAxisItemProps } from './interfaces';
import { useDroppable } from '@dnd-kit/core';
import { SelectAxisSortableItem } from './SelectAxisSortableItem';
import { StylingLabel } from '../../../Common';
import { createStyles } from 'antd-style';
import { SelectAxisContainerId } from '../config';

interface AvailableItemsListProps {
  items: SelectAxisItemProps[];
  activeZone: SelectAxisContainerId | null;
  isActive: boolean;
}

export const AvailableItemsList: React.FC<AvailableItemsListProps> = ({
  items,
  activeZone,
  isActive
}) => {
  const { setNodeRef, isOver } = useDroppable({
    id: SelectAxisContainerId.Available,
    data: {
      type: 'zone'
    }
  });
  const { styles, cx } = useStyles();

  const showDeleteHoverState =
    (isOver || isActive) && activeZone !== SelectAxisContainerId.Available;

  return (
    <div ref={setNodeRef}>
      <StylingLabel label="Available">
        <div className={cx(styles.container, showDeleteHoverState ? 'showDelete' : '')}>
          {items.map((item) => (
            <SelectAxisSortableItem
              key={item.id}
              item={item}
              zoneId={SelectAxisContainerId.Available}
            />
          ))}
        </div>
      </StylingLabel>
    </div>
  );
};

const useStyles = createStyles(({ css, token }) => ({
  container: css`
    &.showDelete {
      box-shadow: 0 0 3px 1px ${token.colorError};
      background: ${token.colorErrorBg};
      border-radius: ${token.borderRadius}px;
    }
  `
}));
