import React, { PropsWithChildren } from 'react';
import {
  BusterMessageData,
  IBusterThread,
  IBusterThreadMessage
} from '@/context/Threads/interfaces';
import { ThreadControllerChart } from './ThreadControllerChart';
import { ChartType, ViewType } from '@/components/charts';
import { ThreadControllerTitleAndControls } from './ThreadControllerTitleAndControls';
import { AppContent } from '@/app/app/_components/AppContent';

export const ThreadControllerChartContent: React.FC<
  PropsWithChildren<{
    thread: IBusterThread;
    currentThreadMessage: IBusterThreadMessage | null;
    currentMessageData: BusterMessageData;
    onSetSelectedChart: (selectedChart: ChartType) => void;
    onSetViewType: (viewType: ViewType) => void;
    onChangeTitle: (title: string) => void;
    isChartEditable: boolean;
    editingTitle: boolean;
    setIsEditingTitle: (editing: boolean) => void;
    isReadOnly: boolean;
    chartOnlyView?: boolean;
  }>
> = ({
  thread,
  currentMessageData,
  onSetSelectedChart,
  onSetViewType,
  currentThreadMessage,
  isChartEditable,
  onChangeTitle,
  editingTitle,
  setIsEditingTitle,
  isReadOnly,
  chartOnlyView
}) => {
  const contentClassName = chartOnlyView ? '' : 'h-full px-8 py-8';

  return (
    <>
      <AppContent className={contentClassName}>
        {currentThreadMessage && (
          <div className="flex h-full w-full flex-col overflow-hidden">
            {!chartOnlyView && (
              <ThreadControllerTitleAndControls
                title={currentThreadMessage.title}
                selectedView={currentThreadMessage.chart_config.selectedView}
                selectedChartType={currentThreadMessage.chart_config.selectedChartType}
                timeFrame={currentThreadMessage.time_frame}
                retrievedData={currentMessageData.retrievedData}
                data={currentMessageData.data}
                description={currentThreadMessage.description}
                isChartEditable={isChartEditable}
                onSetSelectedChart={onSetSelectedChart}
                onSetViewType={onSetViewType}
                onChangeTitle={onChangeTitle}
                editingTitle={editingTitle}
                setIsEditingTitle={setIsEditingTitle}
                isReadOnly={isReadOnly}
                useDropAnimation={thread.isNewThread}
              />
            )}

            <ThreadControllerChart
              datasetId={thread.dataset_id}
              datasetName={thread.dataset_name}
              isChartEditable={isChartEditable}
              currentThreadMessage={currentThreadMessage}
              currentMessageData={currentMessageData}
              chartOnlyView={chartOnlyView}
            />
          </div>
        )}
      </AppContent>
    </>
  );
};
