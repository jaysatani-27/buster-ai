import React, { useMemo } from 'react';
import { DatasetOption } from './useDatasetOptions';

export const useColors = ({
  colors: colorsProp,
  yAxisKeys,
  y2AxisKeys,
  datasetOptions
}: {
  colors: string[];
  yAxisKeys: string[];
  y2AxisKeys: string[];
  datasetOptions: DatasetOption[];
}) => {
  const numberOfYAxisKeys = yAxisKeys.length;
  const numberOfY2AxisKeys = y2AxisKeys.length;
  const totalNumberOfKeys = numberOfYAxisKeys + numberOfY2AxisKeys;
  const lastDatasetOption = datasetOptions[datasetOptions.length - 1];
  const sourceLength = lastDatasetOption.source.length || 1;

  const colors: string[] = useMemo(() => {
    return Array.from(
      { length: totalNumberOfKeys * sourceLength },
      (_, i) => colorsProp[i % colorsProp.length]
    );
  }, [colorsProp, totalNumberOfKeys]);

  return colors;
};
