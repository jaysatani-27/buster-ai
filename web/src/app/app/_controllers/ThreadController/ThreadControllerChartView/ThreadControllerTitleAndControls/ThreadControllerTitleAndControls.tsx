import React, { useMemo } from 'react';
import { ChartType, ViewType } from '@/components/charts';
import { useMemoizedFn, useUnmount } from 'ahooks';
import { AnimatePresence, motion } from 'framer-motion';
import { BusterMessageData, IBusterThreadMessage } from '@/context/Threads';
import isEmpty from 'lodash/isEmpty';
import { ChartSubTitle } from './ChartSubtitle';
import { ChartControls } from './ChartControls';
import { ChartTitle } from './ChartTitle';

export const ThreadControllerTitleAndControls: React.FC<{
  title: IBusterThreadMessage['title'];
  selectedView: IBusterThreadMessage['chart_config']['selectedView'];
  selectedChartType: IBusterThreadMessage['chart_config']['selectedChartType'];
  timeFrame: IBusterThreadMessage['time_frame'];
  retrievedData: BusterMessageData['retrievedData'];
  data: BusterMessageData['data'];
  description: IBusterThreadMessage['description'];
  onSetViewType: (viewType: ViewType) => void;
  onSetSelectedChart: (selectedChart: ChartType) => void;
  onChangeTitle: (title: string) => void;
  isChartEditable: boolean;
  editingTitle: boolean;
  setIsEditingTitle: (editing: boolean) => void;
  isReadOnly: boolean;
  useDropAnimation: boolean;
}> = React.memo(
  ({
    onChangeTitle,
    title,
    selectedView,
    selectedChartType,
    timeFrame,
    retrievedData,
    data,
    description,
    onSetViewType,
    setIsEditingTitle,
    editingTitle,
    isReadOnly,
    useDropAnimation
  }) => {
    const disableControls = !retrievedData || isEmpty(data);
    const isGeneratingTitle = !title;
    const showTitleAndDescriptionContainer = !!title || !!description || !useDropAnimation;
    const titleKey = title || '_title';
    const isTableChart = selectedChartType === ChartType.Table;

    const memoizedTitleAndControlsAnimation = useMemo(
      () => ({
        initial: { opacity: 0, height: 0 },
        animate: {
          opacity: 1,
          minHeight: showTitleAndDescriptionContainer ? '68px' : 0,
          height: showTitleAndDescriptionContainer ? '68px' : 0
        },
        transition: { duration: 0.4, opacity: { delay: 0.2 }, ease: 'easeOut' }
      }),
      [showTitleAndDescriptionContainer]
    );

    const onChangeEditingTitle = useMemoizedFn((v: string) => {
      if (v && v !== title) {
        onChangeTitle(v);
      }
    });

    useUnmount(() => {
      setIsEditingTitle(false);
    });

    return (
      <AnimatePresence mode="wait" initial={!showTitleAndDescriptionContainer}>
        <motion.div
          className={`flex w-full items-start justify-between space-x-4 overflow-hidden`}
          {...memoizedTitleAndControlsAnimation}>
          <AnimatePresence mode="wait" initial={false}>
            <motion.div
              key={titleKey}
              {...titleAnimation}
              className="flex w-full flex-col overflow-hidden">
              <ChartTitle
                title={title}
                isReadOnly={isReadOnly}
                editingTitle={editingTitle}
                setIsEditingTitle={setIsEditingTitle}
                onChangeEditingTitle={onChangeEditingTitle}
                isGeneratingTitle={isGeneratingTitle}
              />
              <ChartSubTitle timeFrame={timeFrame} description={description} />
            </motion.div>
          </AnimatePresence>
          {!isTableChart && (
            <ChartControls
              showTitleAndDescriptionContainer={showTitleAndDescriptionContainer}
              onSetViewType={onSetViewType}
              viewType={selectedView}
              disableControls={disableControls}
            />
          )}
        </motion.div>
      </AnimatePresence>
    );
  }
);
ThreadControllerTitleAndControls.displayName = 'ThreadControllerTitleAndControls';

const titleAnimation = {
  initial: { opacity: 0 },
  animate: { opacity: 1 },
  exit: { opacity: 0 },
  transition: { duration: 0.35 }
};
