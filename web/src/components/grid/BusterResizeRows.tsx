'use client';

import React, { useMemo, useRef, useState } from 'react';
import { BusterResizeableGridRow } from './interfaces';
import { BusterResizeColumns } from './BusterResizeColumns';
import classNames from 'classnames';
import { BusterNewItemDropzone } from './_BusterBusterNewItemDropzone';
import { MAX_HEIGHT_OF_ITEM, MIN_ROW_HEIGHT, TOP_SASH_ID, NEW_ROW_ID } from './config';
import { createStyles } from 'antd-style';
import clamp from 'lodash/clamp';
import { useDebounceFn, useMemoizedFn, useUpdateLayoutEffect } from 'ahooks';
import { useDroppable } from '@dnd-kit/core';

export const BusterResizeRows: React.FC<{
  rows: BusterResizeableGridRow[];
  className: string;
  allowEdit?: boolean;
  onRowLayoutChange: (rows: BusterResizeableGridRow[]) => void;
}> = ({ allowEdit = true, rows, className, onRowLayoutChange }) => {
  const ref = useRef<HTMLDivElement>(null);
  const [isDraggingResizeId, setIsDraggingResizeId] = useState<number | null>(null);
  const [sizes, setSizes] = useState<number[]>(rows.map((r) => r.rowHeight ?? MIN_ROW_HEIGHT));

  const { run: handleRowLayoutChangeDebounced } = useDebounceFn(
    useMemoizedFn((sizes: number[]) => {
      const newRows = rows.map((r, index) => ({
        ...r,
        rowHeight: sizes[index]
      }));
      onRowLayoutChange(newRows);
    }),
    { wait: 375 }
  );

  const handleResize = useMemoizedFn((index: number, size: number) => {
    const newSizes = [...sizes];
    newSizes[index] = size;
    setSizes(newSizes);
    handleRowLayoutChangeDebounced(newSizes);
  });

  const onRowLayoutChangePreflight = useMemoizedFn((columnSizes: number[], rowId: string) => {
    const newRows: BusterResizeableGridRow[] = rows.map((r) => {
      if (r.id === rowId) {
        return { ...r, columnSizes };
      }
      return r;
    });

    onRowLayoutChange(newRows);
  });

  useUpdateLayoutEffect(() => {
    setSizes(rows.map((r) => r.rowHeight ?? MIN_ROW_HEIGHT));
  }, [rows.length]);

  return (
    <div
      ref={ref}
      className={classNames(
        'buster-resize-row relative',
        'mb-10 flex h-full w-full flex-col space-y-3 transition',
        className,
        'opacity-100'
      )}>
      <ResizeRowHandle
        id={TOP_SASH_ID}
        top={true}
        sizes={sizes}
        active={false}
        setIsDraggingResizeId={setIsDraggingResizeId}
        onResize={handleResize}
        allowEdit={allowEdit}
      />

      {rows.map((row, index) => (
        <div
          key={row.id}
          className="relative h-full w-full"
          style={{
            height: sizes[index]
          }}>
          <BusterResizeColumns
            rowId={row.id}
            items={row.items}
            index={index}
            allowEdit={allowEdit}
            columnSizes={row.columnSizes}
            onRowLayoutChange={onRowLayoutChangePreflight}
          />

          <ResizeRowHandle
            id={index.toString()}
            index={index}
            sizes={sizes}
            active={isDraggingResizeId === index}
            setIsDraggingResizeId={setIsDraggingResizeId}
            onResize={handleResize}
            allowEdit={allowEdit}
            hideDropzone={index === rows.length - 1}
          />
        </div>
      ))}

      {allowEdit && <BusterNewItemDropzone />}
    </div>
  );
};

export const useDropzoneStyles = createStyles(({ css, token }) => ({
  dropzone: css`
    transition: background-color 0.2s ease;
    background-color: ${token.colorBorderSecondary};

    &.active {
      opacity: 1;
      z-index: 2;
      background: ${token.colorPrimary} !important;
    }
  `
}));

const useStyles = createStyles(({ css, token }) => ({
  dragger: css`
    transition: background-color 0.2s ease;

    &.dragger {
      cursor: row-resize;
      &:hover {
        background-color: ${token.colorBorder};
      }
    }

    &.active {
      background-color: ${token.colorPrimary} !important;
    }
  `,
  hitArea: css`
    left: 0;
    right: 0;
    height: 54px; // Reduced from 54px to be more reasonable
    position: absolute;
    z-index: 9;
    pointer-events: none;

    // Remove background-color and opacity for production
    // background-color: red;
    opacity: 0;

    // &.active {
    //   opacity: 0;
    // }

    &.top {
      top: -36px; // Position the hit area to straddle the dragger
    }

    &:not(.top) {
      bottom: -15px; // Position the hit area to straddle the dragger
    }
  `
}));

const ResizeRowHandle: React.FC<{
  id: string;
  index?: number;
  sizes: number[];
  setIsDraggingResizeId: (index: number | null) => void;
  onResize: (index: number, size: number) => void;
  allowEdit: boolean;
  active: boolean;
  top?: boolean; //if true we will not use dragging, just dropzone
  hideDropzone?: boolean;
}> = React.memo(
  ({ hideDropzone, top, id, active, allowEdit, setIsDraggingResizeId, index, sizes, onResize }) => {
    const { styles, cx } = useStyles();
    const { styles: dropzoneStyles } = useDropzoneStyles();
    const { setNodeRef, isOver, over } = useDroppable({
      id: `${NEW_ROW_ID}_${id}}`,
      disabled: !allowEdit,
      data: { id }
    });
    const showDropzone = !!over?.id && !hideDropzone;
    const isDropzoneActive = showDropzone && isOver;

    const handler = useMemoizedFn((mouseDownEvent: React.MouseEvent<HTMLDivElement>) => {
      const startPosition = mouseDownEvent.pageY;
      const style = document.createElement('style');
      style.innerHTML = `* { cursor: row-resize; }`;
      document.head.appendChild(style);
      setIsDraggingResizeId(index!);

      function onMouseMove(mouseMoveEvent: MouseEvent) {
        const newSize = sizes[index!] + (mouseMoveEvent.pageY - startPosition);
        const clampedSize = clamp(newSize, MIN_ROW_HEIGHT, MAX_HEIGHT_OF_ITEM);
        onResize(index!, clampedSize);
      }
      function onMouseUp() {
        document.body.removeEventListener('mousemove', onMouseMove);
        style.remove();
        setIsDraggingResizeId(null);
      }

      document.body.addEventListener('mousemove', onMouseMove);
      document.body.addEventListener('mouseup', onMouseUp, { once: true });
    });

    const onMouseDown = top ? undefined : handler;

    const memoizedStyle = useMemo(() => {
      return {
        zIndex: 1,
        bottom: !top ? -4 : -4,
        transform: !top ? 'translateY(100%)' : 'translateY(100%)'
      };
    }, [top]);

    const showActive = (active || isDropzoneActive) && allowEdit;

    return (
      <div className="relative">
        <div
          id={id}
          className={cx(
            showDropzone && allowEdit && dropzoneStyles.dropzone,
            'h-[4px] w-2 w-full select-none rounded',
            allowEdit && styles.dragger,
            !top && 'dragger absolute',
            showActive && 'active'
          )}
          style={memoizedStyle}
          onMouseDown={onMouseDown}
        />
        <div
          className={cx(styles.hitArea, top && 'top', showActive && 'active')}
          ref={setNodeRef}
        />
      </div>
    );
  }
);

ResizeRowHandle.displayName = 'ResizeRowHandle';
