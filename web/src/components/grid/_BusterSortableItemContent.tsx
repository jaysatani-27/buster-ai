import React, { forwardRef } from 'react';
import { createStyles } from 'antd-style';

const useStyles = createStyles(({ css, token }) => ({
  item: css`
    transition: box-shadow transform 200ms ease !important;
    box-shadow: var(--box-shadow);
    --box-shadow: 0px 0px 0px rgb(148 148 148);
    transform: translate3d(var(--translate-x, 0), var(--translate-y, 0), 0) scale(var(--scale, 1));

    &.dragging {
      z-index: 1;
      transition: none !important;
    }

    &.dragOverlay {
      --box-shadow: ${token.boxShadow};
    }
  `
}));

export const BusterSortableItemContent = React.memo(
  forwardRef<
    HTMLDivElement,
    {
      style?: React.CSSProperties;
      itemId: string;
      isDragOverlay?: boolean;
      isDragging?: boolean;
      children?: React.ReactNode;
    }
  >(({ style, children, itemId, isDragging, isDragOverlay }, setNodeRef) => {
    const { cx, styles } = useStyles();

    return (
      <div
        ref={setNodeRef}
        style={style}
        className={cx(
          'relative h-full w-full overflow-hidden rounded',
          styles.item,
          isDragOverlay && 'dragOverlay',
          isDragging && 'dragging',
          isDragging ? (isDragOverlay ? 'opacity-90' : 'opacity-40') : 'opacity-100'
        )}>
        {children}
      </div>
    );
  })
);
BusterSortableItemContent.displayName = 'BusterSortableItemContent';
