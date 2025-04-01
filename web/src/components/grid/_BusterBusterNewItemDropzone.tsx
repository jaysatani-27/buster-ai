import { useDroppable } from '@dnd-kit/core';
import { createStyles } from 'antd-style';
import React from 'react';
import { HEIGHT_OF_DROPZONE, NEW_ROW_ID } from './config';

const useStyles = createStyles(({ token, css }) => ({
  dropzone: css`
    color: ${token.colorTextDescription};
    border-width: 0.5px;
    border-color: ${token.colorBorder};
  `,
  activeDropzone: css`
    background-color: ${token.colorPrimaryBorderHover};
    color: ${token.colorTextLightSolid};
    border-style: solid !important;
  `
}));

export const BusterNewItemDropzone: React.FC = () => {
  const { cx, styles } = useStyles();
  const { setNodeRef, isOver } = useDroppable({
    id: NEW_ROW_ID
  });

  return (
    <div
      ref={setNodeRef}
      style={{ maxHeight: HEIGHT_OF_DROPZONE, minHeight: HEIGHT_OF_DROPZONE }}
      className={cx(
        'flex h-full w-full items-center justify-center rounded border-dashed',
        'transition-colors duration-200 ease-in-out',
        styles.dropzone,
        isOver && styles.activeDropzone
      )}>
      Drag here to create a new row
    </div>
  );
};
