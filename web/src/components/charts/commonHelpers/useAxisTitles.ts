import {
  BusterChartProps,
  ChartEncodes,
  ChartType,
  ComboChartAxis
} from '@/components/charts/interfaces';
import React, { useMemo } from 'react';
import { useXAxisTitle } from './useXAxisTitle';
import { useYAxisTitle } from './useYAxisTitle';
import { useY2AxisTitle } from './useY2AxisTitle';

const emptyY2Axis: string[] = [];

export const useAxisTitles = ({
  selectedAxis,
  columnLabelFormats,
  yAxisShowAxisTitle,
  yAxisAxisTitle,
  xAxisShowAxisTitle,
  xAxisAxisTitle,
  selectedChartType,
  y2AxisShowAxisTitle,
  y2AxisAxisTitle,
  barLayout
}: {
  selectedAxis: ChartEncodes;
  columnLabelFormats: NonNullable<BusterChartProps['columnLabelFormats']>;
  yAxisShowAxisTitle: BusterChartProps['yAxisShowAxisTitle'];
  yAxisAxisTitle: BusterChartProps['yAxisAxisTitle'];
  xAxisShowAxisTitle: BusterChartProps['xAxisShowAxisTitle'];
  xAxisAxisTitle: BusterChartProps['xAxisAxisTitle'];
  selectedChartType: BusterChartProps['selectedChartType'];
  y2AxisShowAxisTitle: BusterChartProps['y2AxisShowAxisTitle'];
  y2AxisAxisTitle: BusterChartProps['y2AxisAxisTitle'];
  barLayout: BusterChartProps['barLayout'];
}) => {
  const yAxis = selectedAxis.y;
  const xAxis = selectedAxis.x;
  const y2Axis = (selectedAxis as ComboChartAxis).y2 || emptyY2Axis;

  const useHorizontalBarLayout = useMemo(() => {
    return barLayout === 'horizontal' && selectedChartType === ChartType.Bar;
  }, [barLayout, selectedChartType]);

  const isSupportedChartForAxisTitles = useMemo(() => {
    const types = [ChartType.Bar, ChartType.Line, ChartType.Scatter, ChartType.Combo];
    return types.includes(selectedChartType);
  }, [selectedChartType]);

  const y2AxisTitle = useY2AxisTitle({
    y2Axis,
    columnLabelFormats,
    isSupportedChartForAxisTitles,
    y2AxisAxisTitle,
    y2AxisShowAxisTitle
  });

  const yAxisTitle = useYAxisTitle({
    yAxis,
    columnLabelFormats,
    isSupportedChartForAxisTitles,
    yAxisAxisTitle,
    yAxisShowAxisTitle,
    selectedAxis
  });

  const xAxisTitle = useXAxisTitle({
    xAxis,
    columnLabelFormats,
    isSupportedChartForAxisTitles,
    xAxisAxisTitle,
    xAxisShowAxisTitle,
    selectedAxis
  });

  return {
    yAxisTitle: useHorizontalBarLayout ? xAxisTitle : yAxisTitle,
    xAxisTitle: useHorizontalBarLayout ? yAxisTitle : xAxisTitle,
    y2AxisTitle
  };
};
