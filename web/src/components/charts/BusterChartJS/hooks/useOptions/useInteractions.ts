import type { DeepPartial } from 'utility-types'; // Add this import
import type { CoreInteractionOptions } from 'chart.js';
import { BusterChartConfigProps } from '@/components/charts/interfaces';
import { useMemo } from 'react';

interface UseInteractionsProps {
  selectedChartType: BusterChartConfigProps['selectedChartType'];
  barLayout: BusterChartConfigProps['barLayout'];
}

export const useInteractions = ({ selectedChartType, barLayout }: UseInteractionsProps) => {
  const interaction: DeepPartial<CoreInteractionOptions> | undefined = useMemo(() => {
    if (selectedChartType === 'scatter') {
      return {
        intersect: true,
        axis: 'xy',
        mode: 'nearest',
        includeInvisible: false
      } as CoreInteractionOptions;
    }

    if (selectedChartType === 'bar' || selectedChartType === 'line') {
      const isHorizontalBar = selectedChartType === 'bar' && barLayout === 'horizontal';
      return {
        intersect: false,
        mode: 'index',
        includeInvisible: false,
        axis: isHorizontalBar ? ('y' as 'y') : 'x'
      } as CoreInteractionOptions;
    }

    if (selectedChartType === 'combo') {
      return {
        intersect: false,
        mode: 'nearest',
        includeInvisible: false,
        axis: 'x'
      } as CoreInteractionOptions;
    }

    return undefined;
  }, [selectedChartType, barLayout]);

  return interaction;
};
