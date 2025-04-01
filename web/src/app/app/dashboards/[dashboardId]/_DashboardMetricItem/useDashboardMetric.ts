import { useDashboardContextSelector } from '@/context/Dashboards';
import { useBusterMessageDataContextSelector } from '@/context/MessageData';
import { useDashboardIndividualContextSelector } from '../_DashboardInvididualContext';
import { useEffect, useRef } from 'react';
import { useInViewport } from 'ahooks';

export const useDashboardMetric = ({ metricId }: { metricId: string }) => {
  const ref = useRef<HTMLDivElement>(null);
  const [inViewport] = useInViewport(ref, {
    threshold: 0.25
  });

  const metric = useDashboardContextSelector(({ getMetric }) => getMetric(metricId));
  const messageData = useBusterMessageDataContextSelector(({ getMessageData }) =>
    getMessageData(metricId)
  );
  const metricMetadata = useDashboardIndividualContextSelector(
    ({ metricMetadata }) => metricMetadata[metricId]
  );

  const setInitialAnimationEnded = useDashboardIndividualContextSelector(
    ({ setInitialAnimationEnded }) => setInitialAnimationEnded
  );

  const setHasBeenScrolledIntoView = useDashboardIndividualContextSelector(
    ({ setHasBeenScrolledIntoView }) => setHasBeenScrolledIntoView
  );

  const initialAnimationEnded = metricMetadata?.initialAnimationEnded || false;
  const renderChart = metricMetadata?.hasBeenScrolledIntoView || false;

  useEffect(() => {
    if (inViewport) {
      setHasBeenScrolledIntoView(metricId);
    }
  }, [inViewport]);

  return { renderChart, metric, ref, messageData, initialAnimationEnded, setInitialAnimationEnded };
};
