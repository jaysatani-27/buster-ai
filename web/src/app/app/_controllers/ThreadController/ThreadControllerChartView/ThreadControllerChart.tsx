import { BusterChart, ViewType } from '@/components/charts';
import React, { useMemo } from 'react';
import { BusterMessageData, IBusterThreadMessage } from '@/context/Threads/interfaces';
import { useUserConfigContextSelector } from '@/context/Users';
import isEmpty from 'lodash/isEmpty';
import { ThreadControllerDatasetLink } from './ThreadControllerDatasetLink';

export const ThreadControllerChart: React.FC<{
  datasetName: string | null;
  datasetId: string | null;
  currentThreadMessage: IBusterThreadMessage;
  currentMessageData: BusterMessageData;
  isChartEditable: boolean;
  className?: string;
  chartOnlyView?: boolean;
}> = React.memo(
  ({
    className = '',
    currentMessageData,
    isChartEditable,
    datasetId,
    datasetName,
    currentThreadMessage,
    chartOnlyView
  }) => {
    const isAnonymousUser = useUserConfigContextSelector((state) => state.isAnonymousUser);
    const chartConfig = currentThreadMessage.chart_config;
    const selectedChartView = chartConfig.selectedView;
    const selectedChartType = chartConfig.selectedChartType;
    const loading = !currentMessageData.retrievedData && isEmpty(currentMessageData.data);
    const selectedData = currentMessageData.dataFromRerun
      ? currentMessageData.dataFromRerun
      : currentMessageData.data;
    const columnMetadata = currentThreadMessage.data_metadata?.column_metadata;
    const isTable = selectedChartView === ViewType.Table || selectedChartType === 'table';

    const chartErrorMessage = useMemo(() => {
      if (currentThreadMessage.error) {
        return currentThreadMessage.error;
      }
      if (currentMessageData.code === null && currentThreadMessage.isCompleted) {
        return 'No code was generated for this request';
      }
      return undefined;
    }, [currentThreadMessage.error, currentMessageData.code, currentThreadMessage.isCompleted]);

    const styles: React.CSSProperties = useMemo(() => {
      if (chartOnlyView) {
        return {
          height: '100%'
        };
      }

      return isTable && !loading && !chartErrorMessage && (selectedData?.length ?? 0) > 0
        ? {
            maxHeight: 'calc(100% - 20px)'
          }
        : {
            height: '100%',
            maxHeight: '600px'
          };
    }, [isTable, chartOnlyView, loading, chartErrorMessage, selectedData?.length]);

    return (
      <div className={`${className} flex h-full w-full flex-col space-y-2.5 overflow-hidden`}>
        <div style={styles} className="w-full overflow-hidden">
          <BusterChart
            data={selectedData!}
            loading={loading}
            error={chartErrorMessage}
            columnMetadata={columnMetadata}
            useRapidResizeObserver={true}
            bordered={!chartOnlyView}
            editable={isChartEditable}
            {...chartConfig}
          />
        </div>

        {!!datasetName && !!datasetId && !isAnonymousUser && (
          <ThreadControllerDatasetLink datasetName={datasetName} datasetId={datasetId} />
        )}
      </div>
    );
  }
);
ThreadControllerChart.displayName = 'ThreadControllerChart';
