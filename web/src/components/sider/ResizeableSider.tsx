import React, { PropsWithChildren, useLayoutEffect } from 'react';
import clamp from 'lodash/clamp';
import { animate } from 'framer-motion';
import { theme } from 'antd';
import { useAntToken } from '@/styles/useAntToken';

export const ResizeableSider: React.FC<
  PropsWithChildren<{
    minDragWidth?: number;
    maxDragWidth?: number;
    minContentWidth?: number;
    initialWidth?: number;
    collapsedWidth?: number;
    collapsed?: boolean;
    onChangeWidth?: (width: number) => void;
    className?: string;
    style?: React.CSSProperties;
  }>
> = ({
  children,
  initialWidth = 220,
  collapsed,
  collapsedWidth = 0,
  minDragWidth = 180,
  maxDragWidth = 320,
  minContentWidth = 180,
  onChangeWidth = () => {},
  className = '',
  style = {}
}) => {
  const token = useAntToken();
  const borderColor = token.colorBorder;
  const siderBg = token.Layout?.siderBg;

  const siderRef = React.useRef<HTMLDivElement>(null);
  const [_width, _setWidth] = React.useState(initialWidth);
  const [savedWidth, setSavedWidth] = React.useState(_width);
  const [isDragging, setIsDragging] = React.useState(false);
  const computedWidth = _width;

  const siderStyle: React.CSSProperties = {
    overflow: 'auto',
    height: '100%',
    minHeight: '100vh',
    transition: 'none',
    backgroundColor: siderBg,
    ...style
  };

  useLayoutEffect(() => {
    if (collapsed) {
      animate(_width, collapsedWidth, {
        duration: 0.15,
        onUpdate: (v) => {
          _setWidth(v);
        }
      });

      setSavedWidth(computedWidth);
    } else {
      animate(computedWidth, savedWidth, {
        duration: 0.15,
        onUpdate: (v) => {
          _setWidth(v);
        }
      });
    }
  }, [collapsed]);

  const resizeableSiderStyle = {
    '--sider-hover-bg': borderColor
  } as React.CSSProperties;

  return (
    <div
      ref={siderRef}
      className={'relative'}
      style={{
        ...siderStyle,
        width: computedWidth
      }}>
      <nav
        className={`h-full max-h-screen min-h-screen w-full ${className}`}
        style={{
          minWidth: minContentWidth
        }}>
        {children}
      </nav>
      <div
        className={`sider-resizer ${isDragging ? 'shadow-buster-slider' : ''} absolute bottom-0 right-0 top-0 h-full cursor-ew-resize pr-[0.5px] transition`}
        // style={{
        //   boxShadow: `inset -1px 0 0 0 ${borderColor}`
        // }}
        onMouseDown={(e) => {
          e.preventDefault();
          const initialWidth = computedWidth;
          const initialX = e.clientX;
          setIsDragging(true);
          const onMouseMove = (e: MouseEvent) => {
            const delta = e.clientX - initialX;
            const newWidth = initialWidth + delta;
            const clampedWidth = clamp(newWidth, minDragWidth, maxDragWidth);
            _setWidth(clampedWidth);
            onChangeWidth(clampedWidth);
            setIsDragging(true);
          };

          const onMouseUp = () => {
            document.removeEventListener('mousemove', onMouseMove);
            document.removeEventListener('mouseup', onMouseUp);
            setIsDragging(false);
          };

          document.addEventListener('mousemove', onMouseMove);
          document.addEventListener('mouseup', onMouseUp);
        }}>
        <div
          className={`h-full w-[2px] transition hover:bg-[var(--sider-hover-bg)] ${isDragging ? 'bg-[var(--sider-hover-bg)]' : ''}`}
          style={resizeableSiderStyle}></div>
      </div>
    </div>
  );
};
