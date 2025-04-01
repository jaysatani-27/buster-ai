import { createStyles } from 'antd-style';
import React, { useMemo } from 'react';
import { Card } from 'antd';
import { useDashboardMetric } from './useDashboardMetric';
import { BusterChart } from '@/components/charts';
import { MetricTitle } from './MetricTitle';
import { createBusterRoute, BusterRoutes } from '@/routes';
import { useMemoizedFn } from 'ahooks';

const _DashboardMetricItem: React.FC<{
  metricId: string;
  dashboardId: string;
  numberOfMetrics: number;
  className?: string;
  isDragOverlay?: boolean;
  allowEdit?: boolean;
}> = ({
  allowEdit,
  dashboardId,
  className = '',
  metricId,
  isDragOverlay = false,
  numberOfMetrics
}) => {
  const { cx, styles } = useStyles();

  const { ref, renderChart, metric, messageData, initialAnimationEnded, setInitialAnimationEnded } =
    useDashboardMetric({ metricId });

  const loadingMetricData = !!metric && !messageData.retrievedData;
  const chartOptions = metric.chart_config;
  const data = messageData.data || null;
  const loading = loadingMetricData;
  const animate = !initialAnimationEnded && !isDragOverlay && numberOfMetrics <= 8;

  const error = useMemo(() => {
    if (metric.error) {
      return metric.error;
    }
    if (metric.code === null) {
      return 'No code was generated for this request';
    }
    return undefined;
  }, [metric]);

  const metricLink = useMemo(() => {
    return createBusterRoute({
      route: BusterRoutes.APP_DASHBOARD_THREADS_ID,
      threadId: metricId,
      dashboardId: dashboardId
    });
  }, [metricId, dashboardId]);

  const onInitialAnimationEndPreflight = useMemoizedFn(() => {
    setInitialAnimationEnded(metricId);
  });

  const cardClassNamesMemoized = useMemo(() => {
    return {
      body: `h-full w-full overflow-hidden !p-0 relative`,
      header: cx(`!p-0 !min-h-[52px]`, styles.cardTitle)
    };
  }, []);

  return (
    <Card
      ref={ref}
      size="small"
      className={`metric-item flex h-full w-full flex-col overflow-auto ${className}`}
      classNames={cardClassNamesMemoized}
      title={
        <MetricTitle
          title={metric?.name}
          timeFrame={metric.time_frame}
          metricLink={metricLink}
          isDragOverlay={isDragOverlay}
          threadId={metricId}
          dashboardId={dashboardId}
          allowEdit={allowEdit}
          description={metric.description}
        />
      }>
      <div
        className={cx(
          'absolute bottom-0 left-0 right-0 top-[1px]',
          `h-full w-full overflow-hidden bg-transparent`,
          isDragOverlay ? 'pointer-events-none' : 'pointer-events-auto'
        )}>
        {renderChart && (
          <BusterChart
            data={data}
            loading={loading}
            error={error}
            bordered={false}
            onInitialAnimationEnd={onInitialAnimationEndPreflight}
            animate={!isDragOverlay && animate}
            columnMetadata={messageData.data_metadata?.column_metadata}
            editable={allowEdit} //this is really only to resize the columns of a table
            {...chartOptions}
          />
        )}
      </div>
    </Card>
  );
};

export const DashboardMetricItem = React.memo(_DashboardMetricItem, (prev, next) => {
  return prev.metricId === next.metricId && prev.dashboardId === next.dashboardId;
});

const useStyles = createStyles(({ token, css }) => ({
  cardTitle: css`
    &:hover {
      background: ${token.controlItemBgHover};
    }
  `
}));
