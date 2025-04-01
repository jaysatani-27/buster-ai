import React, { useMemo, useEffect, useLayoutEffect } from 'react';
import { BusterChartLegendItem, BusterChartLegendProps } from './interfaces';
import { Text, Title } from '@/components/text';
import { createStyles } from 'antd-style';
import { useMemoizedFn } from 'ahooks';
import { LegendItemDot } from './LegendDot';
import { AnimatePresence, motion } from 'framer-motion';

export const LegendItem: React.FC<{
  item: BusterChartLegendItem;
  onClickItem?: BusterChartLegendProps['onClickItem'];
  onFocusItem?: BusterChartLegendProps['onFocusItem'];
  onHoverItem?: BusterChartLegendProps['onHoverItem'];
}> = React.memo(({ item, onClickItem, onFocusItem: onFocusItemProp, onHoverItem }) => {
  const { inactive } = item;

  const onHoverItemPreflight = useMemoizedFn((hover: boolean) => {
    if (!inactive) onHoverItem?.(item, hover);
  });

  const onFocusItemHandler = useMemoizedFn(() => {
    if (onFocusItemProp) onFocusItemProp(item);
  });

  const onFocusItem = onFocusItemProp ? onFocusItemHandler : undefined;

  return (
    <LegendItemStandard
      onClickItem={onClickItem}
      onHoverItemPreflight={onHoverItemPreflight}
      onFocusItem={onFocusItem}
      item={item}
    />
  );
});
LegendItem.displayName = 'LegendItem';

const headlineTypeToText: Record<
  'current' | 'average' | 'total' | 'median' | 'min' | 'max',
  string
> = {
  current: 'Cur.',
  average: 'Avg.',
  total: 'Total',
  median: 'Med.',
  min: 'Min.',
  max: 'Max.'
};

const headlineAnimation = {
  initial: { opacity: 0, height: 0 },
  animate: { opacity: 1, height: '20px' },
  exit: { opacity: 0, height: 0 }
};

const headlinePreTextAnimation = {
  initial: { opacity: 0, width: 0, marginRight: 0 },
  animate: { opacity: 1, width: 'auto', marginRight: '3px' },
  exit: { opacity: 0, width: 0, marginRight: 0 }
};

const LegendItemStandard = React.memo(
  React.forwardRef<
    HTMLDivElement,
    {
      onClickItem: BusterChartLegendProps['onClickItem'];
      onHoverItemPreflight: (hover: boolean) => void;
      onFocusItem: (() => void) | undefined;
      item: BusterChartLegendItem;
    }
  >(({ onClickItem, onHoverItemPreflight, onFocusItem, item }, ref) => {
    const { styles, cx } = useStyles();
    const clickable = onClickItem !== undefined;
    const { formattedName, inactive, headline } = item;
    const hasHeadline = headline !== undefined && headline.type;

    const headlinePreText = useMemo(() => {
      if (hasHeadline && headline.type) return headlineTypeToText[headline.type];
      return '';
    }, [hasHeadline, headline]);

    const onClickItemHandler = useMemoizedFn(() => {
      if (onClickItem) onClickItem(item);
    });

    const onMouseEnterHandler = useMemoizedFn(() => {
      if (onHoverItemPreflight) onHoverItemPreflight(true);
    });

    const onMouseLeaveHandler = useMemoizedFn(() => {
      if (onHoverItemPreflight) onHoverItemPreflight(false);
    });

    const itemWrapperAnimation = useMemo(() => {
      return {
        height: hasHeadline ? 44 : 24,
        borderRadius: hasHeadline ? 8 : 4
      };
    }, [hasHeadline]);

    return (
      <motion.div
        ref={ref}
        initial={false}
        animate={itemWrapperAnimation}
        onClick={onClickItemHandler}
        onMouseEnter={onMouseEnterHandler}
        onMouseLeave={onMouseLeaveHandler}
        className={cx(styles.legendItem, 'flex flex-col justify-center space-y-0', {
          clickable: clickable
        })}>
        <AnimatePresence initial={false}>
          {hasHeadline && (
            <motion.div {...headlineAnimation} className="flex items-center space-x-1.5">
              <Title
                level={4}
                className="!font-semibold leading-none"
                type={!inactive ? 'default' : 'tertiary'}>
                {headline?.titleAmount}
              </Title>
            </motion.div>
          )}
        </AnimatePresence>

        <div
          className={cx('flex flex-nowrap items-center space-x-1.5 whitespace-nowrap', {
            clickable: clickable
          })}>
          <LegendItemDot
            size={!hasHeadline ? 'md' : 'sm'}
            onFocusItem={onFocusItem}
            color={item.color}
            type={item.type}
            inactive={item.inactive}
          />

          <Text
            size="sm"
            className="!flex select-none items-center truncate transition-all duration-100"
            type={!inactive ? 'default' : 'tertiary'}>
            <AnimatePresence mode="wait" initial={false}>
              {headlinePreText && (
                <motion.div
                  key={hasHeadline ? 'hasHeadline' : 'noHeadline'}
                  {...headlinePreTextAnimation}>
                  {headlinePreText}
                </motion.div>
              )}
            </AnimatePresence>

            {formattedName}
          </Text>
        </div>
      </motion.div>
    );
  })
);
LegendItemStandard.displayName = 'LegendItemStandard';

const useStyles = createStyles(({ token, css }) => {
  return {
    legendItem: css`
      padding: 0px 8px;
      border-radius: 4px;
      height: 24px;
      &.clickable {
        transition: background 0.125s ease;
        cursor: pointer;
        &:hover {
          background: ${token.controlItemBgHover};
        }
      }

      &.inactive {
        border-color: ${token.colorBgContainerDisabled};
      }
    `
    // legendItemHeadline: css`
    //   padding: 0px 8px;
    //   border-radius: 8px;
    //   height: 44px;
    //   overflow: hidden;

    //   &.clickable {
    //     cursor: pointer;
    //     transition: background 0.125s ease;
    //     &:hover {
    //       background: ${token.controlItemBgHover};
    //     }
    //   }

    //   &.inactive {
    //     border-color: ${token.colorBgContainerDisabled};
    //   }
    // `
  };
});
