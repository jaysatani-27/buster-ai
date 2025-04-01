import {
  BusterChartProps,
  ChartEncodes,
  ChartType,
  IColumnLabelFormat
} from '@/components/charts/interfaces';
import type { SingleAxisComponentOption } from 'echarts';
import React, { useMemo } from 'react';
import { isNumericColumnType, isNumericColumnStyle } from '@/utils';
import { formatXAxisLabel } from '@/components/charts/commonHelpers';

export const useXAxisTickOptions = ({
  selectedAxis,
  columnLabelFormats,
  columnMetadata,
  selectedChartType,
  useHorizontalBar,
  usePercentageModeAxis,
  xAxisShowAxisLabel,
  xAxisLabelRotation,
  gridLines
}: {
  selectedAxis: ChartEncodes;
  columnLabelFormats: NonNullable<BusterChartProps['columnLabelFormats']>;
  columnMetadata: NonNullable<BusterChartProps['columnMetadata']> | undefined;
  selectedChartType: ChartType;
  useHorizontalBar: boolean;
  usePercentageModeAxis: boolean;
  xAxisShowAxisLabel: NonNullable<BusterChartProps['xAxisShowAxisLabel']>;
  xAxisLabelRotation: NonNullable<BusterChartProps['xAxisLabelRotation']>;
  gridLines: NonNullable<BusterChartProps['gridLines']>;
}) => {
  const isPieChart = selectedChartType === ChartType.Pie;

  const xAxisColumnFormats: Record<string, IColumnLabelFormat> = useMemo(() => {
    return selectedAxis.x.reduce<Record<string, IColumnLabelFormat>>((acc, x) => {
      acc[x] = columnLabelFormats[x];
      return acc;
    }, {});
  }, [selectedAxis.x, columnLabelFormats]);

  const xAxisColumnMetadata = useMemo(() => {
    return columnMetadata?.find((column) => column.name === selectedAxis.x[0]);
  }, [columnMetadata, selectedAxis.x]);

  const xAxis: SingleAxisComponentOption = useMemo(() => {
    if (isPieChart) return {};

    const defaultXAxis: SingleAxisComponentOption = {
      type: 'category',
      axisLabel: {
        show: xAxisShowAxisLabel,
        rotate: xAxisLabelRotation === 'auto' ? undefined : xAxisLabelRotation,
        formatter: (value: string | number) =>
          formatXAxisLabel(
            value,
            selectedAxis,
            xAxisColumnFormats,
            xAxisColumnMetadata,
            selectedChartType
          )
      },
      // axisTick: {
      //   show: xAxisShowTicks
      // },
      splitLine: xAxisSplitLineGenerator(gridLines, selectedChartType)
    };

    return {
      ...defaultXAxis,
      ...xAxisBuilder[selectedChartType]({
        defaultXAxis,
        usePercentageModeAxis,
        xAxisColumnFormats
      })
    };
  }, [
    isPieChart,
    gridLines,
    xAxisColumnMetadata,
    selectedChartType,
    xAxisLabelRotation,
    xAxisShowAxisLabel,
    xAxisColumnFormats,
    useHorizontalBar
  ]);

  return {
    xAxis
  };
};

const xAxisBuilder: Record<
  ChartType,
  (d: {
    defaultXAxis: SingleAxisComponentOption & {
      scale?: boolean;
      boundaryGap?: boolean | [string | number, string | number];
    };
    xAxisColumnFormats: NonNullable<BusterChartProps['columnLabelFormats']>;
    usePercentageModeAxis: boolean;
  }) => Partial<SingleAxisComponentOption>
> = {
  [ChartType.Bar]: ({ defaultXAxis }) => {
    return defaultXAxis;
  },
  [ChartType.Line]: ({ xAxisColumnFormats, usePercentageModeAxis, defaultXAxis }) => {
    if (
      Object.keys(xAxisColumnFormats).length === 1 &&
      xAxisColumnFormats[Object.keys(xAxisColumnFormats)[0]].columnType === 'date'
    ) {
      defaultXAxis.type = 'time';
    }

    if (defaultXAxis.type === 'time') {
      const firstXKey = Object.keys(xAxisColumnFormats)[0];
      const dateFormat = xAxisColumnFormats[firstXKey]?.dateFormat;
      if (dateFormat === 'auto' || !dateFormat) {
        //@ts-ignore //TODO: find what type this actually is
        defaultXAxis.axisLabel = {
          formatter: {
            inherit: true // will inherit the styles from the theme
          }
        };
      }
    } else {
      defaultXAxis.boundaryGap = !usePercentageModeAxis;
    }

    return defaultXAxis;
  },
  [ChartType.Scatter]: ({ xAxisColumnFormats, defaultXAxis }) => {
    const everyXAxisIsNumber = Object.values(xAxisColumnFormats).every(
      (format) => isNumericColumnType(format.columnType) && isNumericColumnStyle(format.style)
    );

    if (everyXAxisIsNumber) {
      defaultXAxis.type = 'value';
      defaultXAxis.scale = true;
    }
    //use category axis style
    else {
      //@ts-ignore
      const xAxisLabelFormatter = defaultXAxis.axisLabel?.formatter;
      defaultXAxis.axisPointer = {
        ...defaultXAxis.axisPointer,
        label: {
          ...defaultXAxis.axisPointer?.label,
          formatter: ({ value }) => xAxisLabelFormatter?.(value)
        }
      };
    }

    return defaultXAxis;
  },
  [ChartType.Pie]: () => ({}),
  [ChartType.Metric]: () => ({}),
  [ChartType.Table]: () => ({}),
  [ChartType.Combo]: () => ({})
};

const xAxisSplitLineGenerator = (gridLines: boolean, selectedChartType: ChartType) => {
  if (selectedChartType === ChartType.Scatter) {
    return {
      show: gridLines
    };
  }

  return {
    show: false
  };
};
