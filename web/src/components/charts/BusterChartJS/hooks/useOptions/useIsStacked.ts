import { ChartType, BusterChartProps } from '@/components/charts/interfaces';
import { useMemo } from 'react';

export const useIsStacked = ({
  selectedChartType,
  lineGroupType,
  barGroupType
}: {
  selectedChartType: ChartType;
  lineGroupType: BusterChartProps['lineGroupType'];
  barGroupType: BusterChartProps['barGroupType'];
}): boolean => {
  return useMemo(() => {
    if (
      selectedChartType === ChartType.Line &&
      (lineGroupType === 'percentage-stack' || lineGroupType === 'stack')
    ) {
      return true;
    }
    if (
      selectedChartType === ChartType.Bar &&
      (barGroupType === 'percentage-stack' || barGroupType === 'stack')
    ) {
      return true;
    }
    return false;
  }, [selectedChartType, lineGroupType, barGroupType]);
};
