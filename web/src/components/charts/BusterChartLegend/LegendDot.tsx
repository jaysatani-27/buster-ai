import { createStyles } from 'antd-style';
import React, { useMemo } from 'react';
import { useMemoizedFn } from 'ahooks';
import { AppMaterialIcons } from '@/components/icons';
import { BusterChartLegendItem } from './interfaces';
import { ChartType } from '../interfaces';

export const LegendItemDot: React.FC<{
  color: string | undefined;
  inactive: boolean;
  type: BusterChartLegendItem['type'];
  onFocusItem?: () => void;
  size?: 'sm' | 'md';
}> = React.memo(({ color, type, inactive, onFocusItem, size = 'md' }) => {
  const { styles, cx } = useStyles();
  const hasFocusItem = onFocusItem !== undefined;

  const onClick = useMemoizedFn((e: React.MouseEvent<HTMLDivElement>) => {
    if (onFocusItem) {
      e.stopPropagation();
      onFocusItem();
    }
  });

  const onFocusItemPreflight = useMemoizedFn((e: React.MouseEvent<HTMLDivElement>) => {
    if (onFocusItem) {
      e.stopPropagation();
      e.preventDefault();
      onFocusItem();
    }
  });

  const dotStyle = useMemo(() => {
    if (type === ChartType.Line) return styles.lineChartDot;
    if (type === ChartType.Scatter) return styles.scatterChartDot;
    return styles.barChartDot;
  }, [type]);

  return (
    <div
      className={cx(
        styles.container,
        size,
        'dot group relative flex items-center justify-center transition-all duration-300'
      )}>
      <div
        onClick={onClick}
        className={cx(dotStyle, styles.dotcontainer, size, 'transition-colors duration-100', {
          inactive,
          'group-hover:opacity-0': hasFocusItem
        })}
        style={{ backgroundColor: !inactive ? color : undefined }}></div>
      {hasFocusItem && (
        <div
          onClick={onFocusItemPreflight}
          className="absolute hidden w-full items-center justify-center overflow-hidden group-hover:flex">
          <div className="focus-item flex h-full w-full items-center justify-center">
            <AppMaterialIcons
              size={size === 'sm' ? 8 : 12}
              className={cx(styles.focusDot, size)}
              icon="target"
            />
          </div>
        </div>
      )}
    </div>
  );
});
LegendItemDot.displayName = 'LegendItemDot';

const useStyles = createStyles(({ token, css }) => {
  return {
    container: css`
      width: 18px;
      height: 12px;

      &.sm {
        width: 8px;
        height: 12px;
      }

      .focus-item {
        border-radius: 4px;
        &:hover {
          background-color: ${token.colorBgElevated} !important;
        }
      }
    `,

    dotcontainer: css`
      background: ${token.colorBorder};
    `,

    barChartDot: css`
      width: 18px;
      height: 12px;
      border-radius: 4px;
      &.sm {
        width: 8px;
        height: 8px;
        border-radius: 1.5px;
      }
    `,
    lineChartDot: css`
      width: 18px;
      height: 4px;
      border-radius: 4px;
      &.sm {
        width: 8px;
        height: 2px;
      }
    `,
    scatterChartDot: css`
      width: 12px;
      height: 12px;
      border-radius: 100%;
      &.sm {
        width: 8px;
        height: 8px;
      }
    `,
    focusDot: css`
      height: 12px;
      width: 12px;
      border-radius: 4px;
      &.sm {
        height: 8px;
        width: 8px;
      }
    `
  };
});
