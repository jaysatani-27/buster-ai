import { AppPopover } from '@/components/tooltip';
import { createStyles } from 'antd-style';
import React from 'react';
import { BusterChartLegendItem, BusterChartLegendProps } from './interfaces';
import { LegendItem } from './LegendItem';
import { Text } from '@/components/text';

export const OverflowButton: React.FC<{
  legendItems: BusterChartLegendItem[];
  onFocusClick?: BusterChartLegendProps['onFocusItem'];
  onClickItem?: BusterChartLegendProps['onClickItem'];
  onHoverItem?: BusterChartLegendProps['onHoverItem'];
}> = React.memo(({ legendItems, onFocusClick, onClickItem, onHoverItem }) => {
  const { styles, cx } = useStyles();

  return (
    <AppPopover
      placement="bottomRight"
      mouseEnterDelay={0.75}
      trigger={['click', 'hover']}
      destroyTooltipOnHide
      className="max-h-[420px] min-w-[200px] !max-w-[265px] overflow-y-auto overflow-x-hidden px-0"
      content={
        <div className="flex flex-col space-y-1 p-0.5">
          {legendItems.map((item) => {
            return (
              <LegendItem
                key={item.id + item.serieName}
                item={item}
                onClickItem={onClickItem}
                onFocusItem={onFocusClick}
                onHoverItem={onHoverItem}
              />
            );
          })}
        </div>
      }>
      <div className={cx('flex items-center space-x-1.5', styles.overflowItemContainer)}>
        <div className={cx(styles.dot, styles.overflowDot, 'flex')} />
        <Text size="sm" className="select-none text-nowrap">
          Next {legendItems.length}
        </Text>
      </div>
    </AppPopover>
  );
});
OverflowButton.displayName = 'OverflowButton';

const useStyles = createStyles(({ token, css }) => {
  return {
    dot: css`
      background: ${token.colorBgContainerDisabled};
      width: 18px;
      min-width: 18px;
      height: 12px;
      border-radius: 4px;

      &.group {
        &:hover {
          &.focus-item {
            background-color: ${token.colorBgElevated} !important;
          }
        }
      }
    `,

    overflowItemContainer: css`
      cursor: pointer;
      padding: 1px 8px;
      border-radius: 4px;
      border: 0px solid ${token.colorBorder};
      // background: ${token.colorBgBase};

      &:hover {
        background: ${token.controlItemBgHover};
      }
    `,
    overflowDot: css`
      background: ${token.colorBorder};
    `,
    overflowItem: css`
      display: flex;
      align-items: center;
      cursor: pointer;
      white-space: nowrap;
      padding: 3px 5px;
      border-radius: 4px;
      &:hover {
        background: ${token.controlItemBgHover};
      }
      &.inactive {
        border-color: ${token.colorBgContainerDisabled};

        .dot {
          background: ${token.colorBgContainerDisabled};
        }
      }
    `
  };
});
