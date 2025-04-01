import React, { useEffect, useMemo, useRef, useState } from 'react';
import { useLegendAutoShow } from './useLegendAutoShow';
import { BusterChartLegendItem } from './interfaces';
import {
  BusterChartProps,
  ChartEncodes,
  ChartType,
  ComboChartAxis,
  ScatterAxis
} from '../interfaces';
import {
  DEFAULT_CATEGORY_AXIS_COLUMN_NAMES,
  DEFAULT_X_AXIS_COLUMN_NAMES,
  DEFAULT_Y_AXIS_COLUMN_NAMES
} from './config';

interface UseBusterChartLegendProps {
  selectedChartType: ChartType;
  showLegendProp: BusterChartProps['showLegend'];
  loading: boolean;
  lineGroupType: BusterChartProps['lineGroupType'];
  barGroupType: BusterChartProps['barGroupType'];
  selectedAxis: ChartEncodes | undefined;
}

export const useBusterChartLegend = ({
  selectedChartType,
  showLegendProp,
  selectedAxis,
  loading,
  lineGroupType,
  barGroupType
}: UseBusterChartLegendProps) => {
  const [inactiveDatasets, setInactiveDatasets] = useState<Record<string, boolean>>({});
  const [legendItems, setLegendItems] = useState<BusterChartLegendItem[]>([]);

  const yAxisColumnNames = selectedAxis?.y ?? DEFAULT_Y_AXIS_COLUMN_NAMES;
  const y2AxisColumnNames = (selectedAxis as ComboChartAxis)?.y2 ?? DEFAULT_Y_AXIS_COLUMN_NAMES;
  const allYAxisColumnNames = useMemo(() => {
    return [...yAxisColumnNames, ...y2AxisColumnNames];
  }, [yAxisColumnNames.join(''), y2AxisColumnNames.join(',')]);

  const xAxisColumnNames = useMemo(
    () => selectedAxis?.x ?? DEFAULT_X_AXIS_COLUMN_NAMES,
    [selectedAxis?.x?.join('')]
  );

  const categoryAxisColumnNames = useMemo(
    () => (selectedAxis as ScatterAxis)?.category ?? DEFAULT_CATEGORY_AXIS_COLUMN_NAMES,
    [(selectedAxis as ScatterAxis)?.category?.join('')]
  );

  const showLegend = useLegendAutoShow({
    selectedChartType,
    showLegendProp,
    categoryAxisColumnNames,
    allYAxisColumnNames
  });

  const renderLegend = useMemo(() => {
    return selectedChartType !== 'metric' && !loading;
  }, [loading, selectedChartType]);

  const isStackPercentage = useMemo(() => {
    return (
      (selectedChartType === 'line' && lineGroupType === 'percentage-stack') ||
      (selectedChartType === 'bar' && barGroupType === 'percentage-stack')
    );
  }, [selectedChartType, lineGroupType, barGroupType]);

  useEffect(() => {
    setInactiveDatasets({});
  }, [allYAxisColumnNames, xAxisColumnNames, categoryAxisColumnNames]);

  return {
    inactiveDatasets,
    setInactiveDatasets,
    legendItems,
    setLegendItems,
    renderLegend,
    isStackPercentage,
    showLegend,
    categoryAxisColumnNames,
    allYAxisColumnNames
  };
};
