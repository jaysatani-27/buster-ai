import React from 'react';
import { DraggableAttributes } from '@dnd-kit/core';
import { SyntheticListenerMap } from '@dnd-kit/core/dist/hooks/utilities';
import { createStyles } from 'antd-style';
import { AppMaterialIcons } from '@/components/icons';

export const SelectAxisItemDragContainer = React.forwardRef<
  HTMLDivElement,
  {
    style?: React.CSSProperties;
    className?: string;
    isDragging?: boolean;
    listeners?: SyntheticListenerMap;
    attributes?: DraggableAttributes;
    children: React.ReactNode;
  }
>(({ style, className = '', children, listeners, attributes, isDragging }, ref) => {
  const { cx, styles } = useStyles();

  return (
    <div
      ref={ref}
      style={style}
      className={cx(
        'flex items-center space-x-1 overflow-hidden rounded',
        styles.container,
        isDragging && 'isDragging cursor-grabbing shadow-lg',
        className
      )}>
      <div
        {...listeners}
        {...attributes}
        className={cx(`flex cursor-grab items-center justify-center`, styles.icon)}>
        <AppMaterialIcons size={20} icon="drag_indicator" />
      </div>
      {children}
    </div>
  );
});

const useStyles = createStyles(({ css, token }) => ({
  container: css`
    height: ${token.controlHeight + 4}px;

    &.isDragging {
      border: 0.5px solid ${token.colorBorder};
      background: ${token.colorBgContainer};
    }
  `,
  icon: css`
    color: ${token.colorIcon};
    width: 32px;
    min-width: 32px;
    height: 100%;
    border-radius: ${token.borderRadius}px;
    cursor: grab;

    &:hover {
      background: ${token.controlItemBgActiveHover};
    }
  `
}));

SelectAxisItemDragContainer.displayName = 'SelectAxisItemDragContainer';
