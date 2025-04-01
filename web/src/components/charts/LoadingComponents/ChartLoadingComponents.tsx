'use client';

import { Text } from '@/components/text';
import { busterChartsTwMerge } from '@/styles/busterChartsTwMerge';
import React from 'react';

export const PreparingYourRequestLoader: React.FC<{
  className?: string;
  text?: string;
  error?: string | null;
}> = ({ className = '', text = 'Processing your request...', error }) => {
  return (
    <div
      className={busterChartsTwMerge(
        'flex h-full w-full items-center justify-center space-x-1.5',
        className
      )}>
      <Text type="tertiary" className="flex items-center text-center">
        {/* {!!error && <AppMaterialIcons icon="error" className="mr-1" />} */}
        {error || text}
      </Text>
    </div>
  );
};

export const NoChartData: React.FC<{
  noDataText?: string;
  className?: string;
}> = ({ className = '', noDataText = 'The query ran successfully but didn’t return any data' }) => {
  return (
    <div
      className={busterChartsTwMerge('flex h-full w-full items-center justify-center', className)}>
      <Text type="tertiary" className={busterChartsTwMerge()}>
        {noDataText}
      </Text>
    </div>
  );
};
