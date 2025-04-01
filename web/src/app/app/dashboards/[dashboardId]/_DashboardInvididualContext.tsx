import React, { useRef, useState } from 'react';
import { useMemoizedFn } from 'ahooks';

import {
  createContext,
  ContextSelector,
  useContextSelector
} from '@fluentui/react-context-selector';

interface DashboardMetricMetadata {
  initialAnimationEnded: boolean;
  hasBeenScrolledIntoView?: boolean;
}

export const useDashboardIndividual = ({}: {}) => {
  const [metricMetadata, setMetricMetadata] = useState<Record<string, DashboardMetricMetadata>>({});

  const setInitialAnimationEnded = useMemoizedFn((id: string) => {
    setMetricMetadata((prev) => ({
      ...prev,
      [id]: {
        ...prev[id],
        initialAnimationEnded: true
      }
    }));
  });

  const setHasBeenScrolledIntoView = useMemoizedFn((id: string) => {
    setMetricMetadata((prev) => ({
      ...prev,
      [id]: {
        ...prev[id],
        hasBeenScrolledIntoView: true
      }
    }));
  });

  return {
    metricMetadata,
    setInitialAnimationEnded,
    setHasBeenScrolledIntoView
  };
};

export const DashboardIndividualPageContext = createContext<
  ReturnType<typeof useDashboardIndividual>
>({} as ReturnType<typeof useDashboardIndividual>);

export const DashboardIndividualProvider = ({ children }: { children: React.ReactNode }) => {
  const value = useDashboardIndividual({});

  return (
    <DashboardIndividualPageContext.Provider value={value}>
      {children}
    </DashboardIndividualPageContext.Provider>
  );
};

export const useDashboardIndividualContextSelector = <T,>(
  selector: ContextSelector<ReturnType<typeof useDashboardIndividual>, T>
) => useContextSelector(DashboardIndividualPageContext, selector);
