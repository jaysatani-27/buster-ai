import { useMemoizedFn } from 'ahooks';
import { useState } from 'react';
import { BusterListContextMenu } from './interfaces';
import React from 'react';

export const useListContextMenu = ({ contextMenu }: { contextMenu?: BusterListContextMenu }) => {
  const [contextMenuPosition, setContextMenuPosition] = useState<{
    x: number;
    y: number;
    scrollYPosition: number;
    show: boolean;
    id: string;
  } | null>(null);

  const onContextMenuClick = useMemoizedFn((e: React.MouseEvent<HTMLDivElement>, id: string) => {
    if (!contextMenu) return;
    e.stopPropagation();
    e.preventDefault();
    const x = e.clientX - 5;
    const y = e.clientY - 5; // offset the top by 30px
    const menuWidth = 250; // width of the menu
    const menuHeight = 200; // height of the menu
    const pageWidth = window.innerWidth;
    const pageHeight = window.innerHeight;

    // Ensure the menu does not render offscreen horizontally
    const adjustedX = Math.min(Math.max(0, x), pageWidth - menuWidth);
    // Ensure the menu does not render offscreen vertically, considering the offset
    const adjustedY = Math.min(Math.max(0, y), pageHeight - menuHeight);

    setContextMenuPosition({
      x: adjustedX,
      y: adjustedY,
      show: true,
      id: id,
      scrollYPosition: window.scrollY
    });
  });

  return {
    contextMenuPosition,
    onContextMenuClick,
    setContextMenuPosition
  };
};
