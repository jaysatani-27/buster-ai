import React, { useMemo } from 'react';

import { Text } from '@/components';
import { useMount } from 'ahooks';
import { BusterChartProps, ChartType } from '../interfaces';

export const NoValidAxis: React.FC<{
  type: ChartType;
  onReady?: () => void;
  data: BusterChartProps['data'];
}> = ({ onReady, type, data }) => {
  const inValidChartText = useMemo(() => {
    if (!type) return 'No valid chart type';
    return 'No valid axis selected';
  }, [type, data]);

  useMount(() => {
    onReady?.();
  });

  return (
    <div className="flex h-full w-full items-center justify-center">
      <Text type="tertiary">{inValidChartText}</Text>
    </div>
  );
};
