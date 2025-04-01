import { ViewType } from '@/components/charts';
import { AnimatePresence, motion } from 'framer-motion';
import React, { useMemo } from 'react';
import { ChartControlsViewType } from '../ChartControlsViewType';

const memoizedAnimation = {
  initial: { opacity: 0 },
  animate: { opacity: 1 },
  exit: { opacity: 0 },
  transition: { duration: 0.25 }
};

export const ChartControls: React.FC<{
  showTitleAndDescriptionContainer: boolean;
  onSetViewType: (viewType: ViewType) => void;
  viewType: ViewType | undefined;
  disableControls: boolean;
}> = React.memo(
  ({ showTitleAndDescriptionContainer, onSetViewType, viewType, disableControls }) => {
    return (
      <AnimatePresence mode="wait" initial={!showTitleAndDescriptionContainer}>
        <motion.div {...memoizedAnimation} className="flex h-full items-start space-x-1">
          <ChartControlsViewType
            onChangeView={onSetViewType}
            viewType={viewType}
            disabled={disableControls}
          />
        </motion.div>
      </AnimatePresence>
    );
  }
);
ChartControls.displayName = 'ChartControls';
