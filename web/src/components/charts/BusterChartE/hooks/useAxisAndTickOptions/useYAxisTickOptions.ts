import { formatYAxisLabel, yAxisSimilar } from '@/components/charts/commonHelpers';
import {
  BusterChartProps,
  ChartType,
  ComboChartAxis,
  IColumnLabelFormat
} from '@/components/charts/interfaces';
import type { SingleAxisComponentOption, YAXisComponentOption } from 'echarts';
import React, { useMemo } from 'react';

const defaultYAxis2AxisKeys: string[] = [];
const defaultYAxis2: YAXisComponentOption[] = [];

export const useYAxisTickOptions = ({
  selectedAxis,
  columnLabelFormats,
  selectedChartType,
  columnSettings,
  yAxisScaleType,
  yAxisStartAxisAtZero,
  yAxisShowAxisLabel,
  y2AxisScaleType,
  y2AxisStartAxisAtZero,
  y2AxisShowAxisLabel,
  gridLines,
  barShowTotalAtTop,
  useHorizontalBar,
  usePercentageModeAxis
}: {
  selectedAxis: ComboChartAxis;
  columnLabelFormats: NonNullable<BusterChartProps['columnLabelFormats']>;
  selectedChartType: ChartType;
  columnSettings: NonNullable<BusterChartProps['columnSettings']>;
  yAxisScaleType: BusterChartProps['yAxisScaleType'];
  yAxisStartAxisAtZero: BusterChartProps['yAxisStartAxisAtZero'];
  yAxisShowAxisLabel: BusterChartProps['yAxisShowAxisLabel'];
  y2AxisScaleType: BusterChartProps['y2AxisScaleType'];
  y2AxisStartAxisAtZero: BusterChartProps['y2AxisStartAxisAtZero'];
  y2AxisShowAxisLabel: BusterChartProps['y2AxisShowAxisLabel'];
  gridLines: NonNullable<BusterChartProps['gridLines']>;
  barShowTotalAtTop: BusterChartProps['barShowTotalAtTop'];
  useHorizontalBar: boolean;
  usePercentageModeAxis: boolean;
}) => {
  const isPieChart = selectedChartType === ChartType.Pie;
  const isComboChart = selectedChartType === ChartType.Combo;
  const yAxisKeys = selectedAxis.y;
  const y2AxisKeys = (selectedAxis as ComboChartAxis).y2 || defaultYAxis2AxisKeys;

  const canUseSameYFormatter = useMemo(() => {
    const hasMultipleY = selectedAxis.y.length > 1;
    return hasMultipleY ? yAxisSimilar(selectedAxis.y, columnLabelFormats) : true;
  }, [selectedAxis.y, columnLabelFormats]);

  const yAxisColumnFormats: Record<string, IColumnLabelFormat> = useMemo(() => {
    return selectedAxis.y.reduce<Record<string, IColumnLabelFormat>>((acc, y) => {
      acc[y] = columnLabelFormats[y];
      return acc;
    }, {});
  }, [selectedAxis.y, columnLabelFormats]);

  const y2AxisColumnFormats: Record<string, IColumnLabelFormat> = useMemo(() => {
    return y2AxisKeys.reduce<Record<string, IColumnLabelFormat>>((acc, y) => {
      acc[y] = columnLabelFormats[y];
      return acc;
    }, {});
  }, [y2AxisKeys, columnLabelFormats]);

  const yAxis: YAXisComponentOption[] = useMemo(() => {
    if (isPieChart) return [];

    const isLogAxis = yAxisScaleType === 'log';

    return [
      {
        type: isLogAxis ? 'log' : 'value',
        max: usePercentageModeAxis ? 100 : undefined,
        min: isLogAxis
          ? null
          : () => {
              if (usePercentageModeAxis || yAxisStartAxisAtZero) return 0;
              return undefined;
            },
        splitLine: yAxisSplitLineGenerator(gridLines, selectedChartType),
        minorSplitLine: {
          show: isLogAxis
        },
        axisLabel: {
          show: yAxisShowAxisLabel,
          formatter: (value: string) =>
            formatYAxisLabel(
              value,
              yAxisKeys,
              canUseSameYFormatter,
              yAxisColumnFormats,
              usePercentageModeAxis
            )
        },
        ...(yAxisBuilder[selectedChartType]({ barShowTotalAtTop, useHorizontalBar }) as any)
      }
    ];
  }, [
    isPieChart,
    gridLines,
    yAxisStartAxisAtZero,
    yAxisShowAxisLabel,
    yAxisKeys,
    selectedChartType,
    canUseSameYFormatter,
    yAxisColumnFormats,
    yAxisScaleType,
    useHorizontalBar,
    usePercentageModeAxis
  ]);

  const y2Axis: YAXisComponentOption[] = useMemo(() => {
    if (!isComboChart || y2AxisKeys.length === 0) return defaultYAxis2;

    const isLogAxis = y2AxisScaleType === 'log';

    return [
      {
        type: isLogAxis ? 'log' : 'value',
        max: usePercentageModeAxis ? 100 : undefined,
        min: isLogAxis
          ? () => 0
          : (v: { min: number; max: number }) => {
              if (usePercentageModeAxis || y2AxisStartAxisAtZero) return 0;
              return v.min - v.min * 0.05;
            },
        splitLine: yAxisSplitLineGenerator(gridLines, selectedChartType),
        axisLabel: {
          show: y2AxisShowAxisLabel,
          formatter: (value: string) =>
            formatYAxisLabel(value, y2AxisKeys, canUseSameYFormatter, y2AxisColumnFormats, false)
        }
      }
    ];
  }, [
    isComboChart,
    y2AxisScaleType,
    y2AxisShowAxisLabel,
    y2AxisStartAxisAtZero,
    y2AxisColumnFormats,
    y2AxisKeys
  ]);

  return {
    yAxis,
    y2Axis
  };
};

const yAxisSplitLineGenerator = (
  gridLines: boolean,
  selectedChartType: ChartType
): YAXisComponentOption['splitLine'] => {
  return {
    show: gridLines
  };
};

const yAxisBuilder: Record<
  ChartType,
  ({
    useHorizontalBar
  }: {
    useHorizontalBar: boolean;
    barShowTotalAtTop: BusterChartProps['barShowTotalAtTop'];
  }) => Partial<SingleAxisComponentOption>
> = {
  [ChartType.Bar]: ({ barShowTotalAtTop }) => ({
    boundaryGap: barShowTotalAtTop ? ['0%', '3%'] : undefined
  }),
  [ChartType.Line]: ({}) => ({}),
  [ChartType.Scatter]: () => ({
    scale: true
  }),
  [ChartType.Pie]: () => ({}),
  [ChartType.Metric]: () => ({}),
  [ChartType.Table]: () => ({}),
  [ChartType.Combo]: () => ({})
};
