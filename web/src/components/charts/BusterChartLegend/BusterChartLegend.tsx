'use client';

import React, { useMemo, useRef, useCallback, useState } from 'react';
import { BusterChartLegendItem, BusterChartLegendProps } from './interfaces';
import { LegendItem } from './LegendItem';
import { OverflowButton } from './OverflowContainer';
import { AnimatePresence, motion } from 'framer-motion';
import { useMemoizedFn } from 'ahooks';
import isEmpty from 'lodash/isEmpty';
import { computeHiddenShowItems } from './helpers';

const MeasureContainer = ({ children }: { children: React.ReactNode }) => {
  return <div className="invisible absolute mt-12 flex w-full items-center">{children}</div>;
};

interface ItemMeasurement {
  width: number;
  height: number;
}

export const BusterChartLegend: React.FC<BusterChartLegendProps> = React.memo(
  ({
    legendItems,
    onClickItem,
    animate,
    containerWidth = 400,
    showLegendHeadline,
    onFocusItem,
    onHoverItem,
    show
  }) => {
    const legendWidth = containerWidth;
    const completedInitialAnimation = useRef(false);

    const { shownItems, hiddenItems } = useMemo(() => {
      if (!show || !legendItems || legendItems.length === 0) {
        return { shownItems: [], hiddenItems: [] };
      }

      const { shownItems, hiddenItems } = computeHiddenShowItems(legendItems, legendWidth);

      return { shownItems, hiddenItems };
    }, [legendItems, legendWidth, show]);

    const legendKey = useMemo(() => {
      return legendItems.map((item) => item.id).join('');
    }, [legendItems]);

    const hasOverflowButtons = hiddenItems.length > 0;
    const showLegend = show && (shownItems.length >= 1 || hasOverflowButtons);
    const onClickItemHandler = legendItems.length > 1 ? onClickItem : undefined;
    const onFocusItemHandler = legendItems.length > 2 ? onFocusItem : undefined;
    const onHoverItemHandler = legendItems.length > 1 ? onHoverItem : undefined;

    const memoizedChildAnimation = useMemo(() => {
      if (!animate) return {};

      return {
        initial: { opacity: 0 },
        animate: { opacity: 1 },
        exit: { opacity: 0 },
        transition: { duration: 0.25 }
      };
    }, [animate, showLegend]);

    const initialHeight = useMemo(() => {
      const hasHeadline = !completedInitialAnimation.current
        ? showLegendHeadline
        : legendItems.some((item) => item.headline);
      if (hasHeadline) return '44px';
      return '24px';
    }, [legendItems, showLegendHeadline]);

    const memoizedAnimation = useMemo(() => {
      if (!animate) return {};
      return {
        initial: {
          height: show && !completedInitialAnimation.current ? initialHeight : 0,
          minHeight: show && !completedInitialAnimation.current ? initialHeight : 0
        },
        animate: {
          height: showLegend ? initialHeight : 0,
          minHeight: !completedInitialAnimation.current
            ? show
              ? initialHeight
              : 0
            : showLegend
              ? initialHeight
              : 0
        },
        exit: { height: 0 },
        transition: { duration: 0.25 }
      };
    }, [animate, initialHeight, showLegend]);

    const onAnimationComplete = useMemoizedFn(() => {
      setTimeout(() => {
        completedInitialAnimation.current = true;
      }, 250 * 1.5);
    });

    return (
      <>
        <motion.div
          className="chart-legend flex w-full items-center overflow-hidden"
          onAnimationComplete={onAnimationComplete}
          {...memoizedAnimation}>
          <AnimatePresence mode="wait" initial={false}>
            {showLegend && (
              <motion.div
                key={legendKey}
                {...memoizedChildAnimation}
                className="flex w-full flex-nowrap justify-end space-x-2 overflow-hidden">
                {shownItems.map((item) => (
                  <LegendItem
                    key={item.id + item.serieName}
                    item={item}
                    onClickItem={onClickItemHandler}
                    onFocusItem={onFocusItemHandler}
                    onHoverItem={onHoverItemHandler}
                  />
                ))}

                {hasOverflowButtons && (
                  <OverflowButton
                    legendItems={hiddenItems}
                    onFocusClick={onFocusItem}
                    onClickItem={onClickItem}
                    onHoverItem={onHoverItem}
                  />
                )}
              </motion.div>
            )}
          </AnimatePresence>
        </motion.div>
      </>
    );
  }
);

BusterChartLegend.displayName = 'BusterChartLegend';
