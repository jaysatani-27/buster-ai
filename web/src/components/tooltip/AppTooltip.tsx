import React, { PropsWithChildren, useCallback, useMemo, useState } from 'react';
import { Tooltip } from 'antd';
import type { TooltipProps } from 'antd';
import { AppTooltipShortcutPill } from './AppTooltipShortcutPill';

export type AppTooltipProps = TooltipProps & {
  shortcuts?: string[];
  title?: string;
  forceShow?: boolean;
  performant?: boolean;
};

export const AppTooltip = React.memo<PropsWithChildren<AppTooltipProps>>(
  ({ children, ...props }) => {
    const [isHovering, setIsHovering] = useState(false);
    const { title, shortcuts = [], forceShow, performant } = props;

    if (!title && !shortcuts.length && !forceShow) {
      return <>{children}</>;
    }

    if (performant && !isHovering) {
      return <PerformanceTooltip setIsHovering={setIsHovering}>{children}</PerformanceTooltip>;
    }

    return <BusterTooltip {...props}>{children}</BusterTooltip>;
  }
);
AppTooltip.displayName = 'AppTooltip';

const PerformanceTooltip: React.FC<
  PropsWithChildren<{
    setIsHovering: (v: boolean) => void;
  }>
> = React.memo(({ children, setIsHovering }) => {
  return (
    <div
      className="performance-tooltip flex"
      onMouseEnter={() => {
        setIsHovering(true);
      }}>
      {children}
    </div>
  );
});
PerformanceTooltip.displayName = 'PerformanceTooltip';

const BusterTooltip: React.FC<PropsWithChildren<AppTooltipProps>> = ({
  children,
  mouseEnterDelay = 0.75,
  mouseLeaveDelay = 0.15,
  destroyTooltipOnHide = true,
  trigger = 'hover',
  arrow = false,
  title = '',
  forceShow = false,
  performant = false,
  shortcuts = [], //⌘
  align,
  ...props
}) => {
  const memoizedTitle = useMemo(() => {
    if (title) {
      return (
        <div className="flex items-center space-x-1">
          <span className="text-sm">{title}</span>
          <AppTooltipShortcutPill shortcut={shortcuts} />
        </div>
      );
    }
    return null;
  }, [title, shortcuts]);

  return (
    <Tooltip
      {...props}
      title={memoizedTitle}
      mouseEnterDelay={mouseEnterDelay}
      trigger={trigger}
      arrow={arrow}
      mouseLeaveDelay={mouseLeaveDelay}>
      {children}
    </Tooltip>
  );
};
BusterTooltip.displayName = 'BusterTooltip';
