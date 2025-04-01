import React, { useMemo, useRef } from 'react';
import { renderToString } from 'react-dom/server';
import { BusterChartProps, ChartEncodes, ChartType, ComboChartAxis } from '../../../interfaces';
import { TooltipFormatterParams } from './interfaces';
import { useMemoizedFn } from 'ahooks';
import type { TooltipComponentOption } from 'echarts';
import {
  DEFAULT_CATEGORY_AXIS_COLUMN_NAMES,
  DEFAULT_Y_AXIS_COLUMN_NAMES
} from '../../../BusterChartLegend/config';
import { BusterEChartTooltip } from './BusterEChartTooltip';
import { extractFieldsFromChain } from '@/components/charts/chartHooks';

const CACHE_DURATION = 3500;

export const useTooltipOptions = ({
  columnLabelFormats,
  selectedChartType,
  tooltipKeys,
  barGroupType,
  lineGroupType,
  pieDisplayLabelAs,
  columnSettings,
  selectedAxis
}: {
  columnLabelFormats: NonNullable<BusterChartProps['columnLabelFormats']>;
  columnSettings: NonNullable<BusterChartProps['columnSettings']>;
  selectedChartType: NonNullable<BusterChartProps['selectedChartType']>;
  tooltipKeys: string[];
  barGroupType: BusterChartProps['barGroupType'];
  lineGroupType: BusterChartProps['lineGroupType'];
  pieDisplayLabelAs: BusterChartProps['pieDisplayLabelAs'];
  selectedAxis: ChartEncodes;
}) => {
  const isComboChart = selectedChartType === ChartType.Combo;

  const useGlobalPercentage = useMemo(() => {
    if (selectedChartType === 'pie') return pieDisplayLabelAs === 'percent';
    if (selectedChartType === 'bar') return barGroupType === 'percentage-stack';
    if (selectedChartType === 'line') return lineGroupType === 'percentage-stack';
    return false;
  }, [lineGroupType, pieDisplayLabelAs, selectedChartType, barGroupType]);

  const yAxisKeys = selectedAxis.y;
  const y2AxisKeys = (selectedAxis as ComboChartAxis)?.y2 || DEFAULT_Y_AXIS_COLUMN_NAMES;
  const categoryAxisKeys =
    (selectedAxis as ComboChartAxis)?.category || DEFAULT_CATEGORY_AXIS_COLUMN_NAMES;

  const keyToUsePercentage = useMemo(() => {
    if (useGlobalPercentage)
      return tooltipKeys.filter((key) => {
        const extracted = extractFieldsFromChain(key).at(-1)?.key!;
        const columnLabelFormat = columnLabelFormats[extracted];
        return columnLabelFormat.columnType === 'number';
      });
    return [];
  }, [useGlobalPercentage, columnSettings, tooltipKeys]);

  const hasTooltipKeys = tooltipKeys.length > 0;

  const useSeriesKeyValueAsLabel = useMemo(() => {
    const allYAxis = [...yAxisKeys, ...y2AxisKeys];
    return allYAxis.length > 1 && categoryAxisKeys.length !== 0;
  }, [yAxisKeys, y2AxisKeys, categoryAxisKeys]);

  // Because the tooltip is re-rendered on every change, we need to cache the result to prevent unnecessary re-renders
  const tooltipCache = useRef({
    seriesType: '',
    seriesIndex: 0,
    dataIndex: 0,
    seriesLength: 0,
    timestamp: 0,
    result: '',
    firstYValue: 0
  });

  const tooltipCacheCheck = useMemoizedFn((params: TooltipFormatterParams[], now: number) => {
    const firstParam = params[0];

    return (
      // Return cached result if within 1000ms and params haven't changed
      now - tooltipCache.current.timestamp < CACHE_DURATION &&
      tooltipCache.current.seriesIndex === firstParam.seriesIndex &&
      tooltipCache.current.seriesType === firstParam.seriesType &&
      tooltipCache.current.dataIndex === firstParam.dataIndex &&
      tooltipCache.current.seriesLength === params.length &&
      tooltipCache.current.firstYValue === firstParam.value[1]
    );
  });

  const toolTipFormatter = useMemoizedFn(
    (params: TooltipFormatterParams[] | TooltipFormatterParams) => {
      const now = Date.now();
      const arrayFormat = Array.isArray(params) ? params : [params];
      const firstParam = arrayFormat[0];

      if (tooltipCacheCheck(arrayFormat, now)) {
        return tooltipCache.current.result;
      }

      // Generate new tooltip content
      const result = renderToString(
        <BusterEChartTooltip
          params={params}
          columnLabelFormats={columnLabelFormats}
          tooltipKeys={tooltipKeys}
          useSeriesKeyValueAsLabel={useSeriesKeyValueAsLabel}
          keyToUsePercentage={keyToUsePercentage}
          useComboLegendDots={isComboChart}
        />
      );

      // Update cache
      tooltipCache.current = {
        result,
        seriesIndex: firstParam.seriesIndex,
        seriesType: firstParam.seriesType,
        dataIndex: firstParam.dataIndex,
        seriesLength: arrayFormat.length,
        firstYValue: firstParam.value[1] as number,
        timestamp: now
      };

      return result;
    }
  );

  const tooltipOptions: TooltipComponentOption = useMemo(() => {
    return {
      show: tooltipKeys.length > 0,
      formatter: toolTipFormatter,
      appendToBody: true,
      //   confine: false, //will confine to the chart container
      ...tooltipBuilders[selectedChartType]()
    };
  }, [hasTooltipKeys, selectedChartType]);

  return tooltipOptions;
};

const tooltipBuilders: Record<ChartType, () => any> = {
  [ChartType.Line]: () => ({
    trigger: 'axis',
    axisPointer: {
      type: 'line'
    }
  }),
  [ChartType.Bar]: () => ({
    trigger: 'axis',
    axisPointer: {
      type: 'shadow'
    }
  }),
  [ChartType.Scatter]: () => ({
    trigger: 'item',
    axisPointer: {
      type: 'cross' //line, none, shadow, cross,
    },
    label: {
      show: true
    }
  }),
  [ChartType.Pie]: () => ({
    trigger: 'item'
  }),
  [ChartType.Metric]: () => ({}),
  [ChartType.Table]: () => ({}),
  [ChartType.Combo]: () => ({})
};
