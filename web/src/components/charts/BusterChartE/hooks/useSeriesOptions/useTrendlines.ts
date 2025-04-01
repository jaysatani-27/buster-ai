'use client';

import { ChartType } from '@/components/charts/interfaces/enum';
import type { LineSeriesOption, SeriesOption } from 'echarts';
import React, { useMemo } from 'react';
import isEmpty from 'lodash/isEmpty';
import { extractFieldsFromChain } from '../../../chartHooks';
import {
  ChartEncodes,
  ColumnLabelFormat,
  IColumnLabelFormat,
  Trendline
} from '@/components/charts/interfaces';
import { formatLabel } from '@/utils';
import { DATASET_IDS } from '../../../chartHooks';
import type { TrendlineDataset } from '../../../chartHooks';
import busterLightTheme from '../useEChartsTheme/buster_light_theme';

export const useTrendlines = ({
  selectedChartType,
  seriesOptions,
  columnLabelFormats = {},
  selectedAxis,
  dataTrendlineOptions
}: {
  selectedChartType: ChartType;
  seriesOptions: SeriesOption[];
  columnLabelFormats: Record<string, IColumnLabelFormat>;
  selectedAxis: ChartEncodes;
  dataTrendlineOptions: TrendlineDataset[];
}) => {
  const yAxisColumnIds = selectedAxis.y;
  const canSupportTrendlines: boolean = useMemo(() => {
    const isValidChartType =
      selectedChartType === ChartType.Line ||
      selectedChartType === ChartType.Bar ||
      selectedChartType === ChartType.Scatter ||
      selectedChartType === ChartType.Combo;

    return isValidChartType && !!yAxisColumnIds.length && !!dataTrendlineOptions?.length;
  }, [selectedChartType, selectedAxis, dataTrendlineOptions?.length]);

  const filteredTrendlines = useMemo(() => {
    if (!canSupportTrendlines) return [];

    let myFilteredTrendlines: Trendline[] = dataTrendlineOptions || [];
    if (selectedChartType === 'bar') {
      const allowedTrendlines: Trendline['type'][] = [
        'average',
        'median',
        'max',
        'min',
        'linear_regression'
      ];
      myFilteredTrendlines =
        dataTrendlineOptions?.filter((trendline) => allowedTrendlines.includes(trendline.type)) ||
        [];
    }

    return myFilteredTrendlines?.filter(
      ({ show, columnId }) => show && columnId && yAxisColumnIds.includes(columnId)
    );
  }, [dataTrendlineOptions, yAxisColumnIds, selectedChartType, canSupportTrendlines]);

  const trendlineSeries = useMemo(() => {
    const canUse = canSupportTrendlines && filteredTrendlines && !isEmpty(filteredTrendlines);
    if (!canUse) return [];
    let trendlineSeriesArray: SeriesOption[] = [];
    seriesOptions.forEach((series) => {
      const { name } = series!;
      const { key } = extractFieldsFromChain(name as string)[0];
      const trendlines = filteredTrendlines.filter((trendline) => trendline.columnId === key);

      if (!trendlines || isEmpty(trendlines)) return series;

      trendlines.forEach((trendline) => {
        const columnLabelFormat = columnLabelFormats[trendline.columnId];
        const newSeries = CreateTrendBuilderRecord[trendline.type]?.(
          trendline,
          series,
          columnLabelFormat,
          dataTrendlineOptions
        );

        if (newSeries) {
          trendlineSeriesArray.push(newSeries);
        }
      });
    });
    return trendlineSeriesArray;
  }, [
    seriesOptions,
    dataTrendlineOptions,
    filteredTrendlines,
    yAxisColumnIds,
    canSupportTrendlines
  ]);

  return [...seriesOptions, ...trendlineSeries];
};

const CreateTrendBuilderRecord: Record<
  Trendline['type'],
  (
    trendline: Trendline,
    seriesOption: SeriesOption,
    columnLabelFormat: IColumnLabelFormat,
    dataTrendlineOptions: TrendlineDataset[]
  ) => void | LineSeriesOption
> = {
  linear_regression: (trendline, seriesOption, columnLabelFormat, dataTrendlineOptions) => {
    return defaultTrendlineFromDataset(trendline, dataTrendlineOptions, 'linear_regression');
  },
  logarithmic_regression: (trendline, seriesOption, columnLabelFormat, dataTrendlineOptions) => {
    return defaultTrendlineFromDataset(trendline, dataTrendlineOptions, 'logarithmic_regression');
  },
  exponential_regression: (trendline, seriesOption, columnLabelFormat, dataTrendlineOptions) => {
    return defaultTrendlineFromDataset(trendline, dataTrendlineOptions, 'exponential_regression');
  },
  polynomial_regression: (trendline, seriesOption, columnLabelFormat, dataTrendlineOptions) => {
    return defaultTrendlineFromDataset(trendline, dataTrendlineOptions, 'polynomial_regression');
  },
  min: (trendline, seriesOption, columnLabelFormat) => {
    medianMaxMinFormatter('min', trendline, columnLabelFormat, seriesOption);
  },
  max: (trendline, seriesOption, columnLabelFormat) => {
    medianMaxMinFormatter('max', trendline, columnLabelFormat, seriesOption);
  },
  median: (trendline, seriesOption, columnLabelFormat) => {
    medianMaxMinFormatter('median', trendline, columnLabelFormat, seriesOption);
  },
  average: (trendline, seriesOption, columnLabelFormat) => {
    medianMaxMinFormatter('average', trendline, columnLabelFormat, seriesOption);
  }
};

const medianMaxMinFormatter = (
  operation: 'max' | 'min' | 'median' | 'average',
  trendline: Trendline,
  columnLabelFormat: ColumnLabelFormat,
  seriesOption: SeriesOption
) => {
  const { showTrendlineLabel, type, trendLineColor, trendlineLabel } = trendline;
  seriesOption.markLine = {
    ...seriesOption.markLine,
    lineStyle: {
      color: trendLineColor || undefined
    },
    data: [
      ...(seriesOption.markLine?.data || []),
      {
        type: operation,
        label: {
          show: showTrendlineLabel,
          formatter: ({ value }: { value: string | number }) => {
            return defaultLabelFormatter(trendlineLabel, value, type, columnLabelFormat);
          }
        }
      }
    ]
  };
};

const defaultLabelFormatter = (
  trendlineLabel: string | null,
  value: string | number,
  type: Trendline['type'],
  columnLabelFormat: ColumnLabelFormat
) => {
  if (trendlineLabel === null) {
    const formattedValue = formatLabel(value, columnLabelFormat, false);
    const formattedLabel = formatLabel(type, undefined, true);
    return `${formattedLabel}: ${formattedValue}`;
  }
  return trendlineLabel;
};

const getSearchId = (type: Trendline['type'], columnId: string) => {
  if (type === 'linear_regression') return DATASET_IDS.linearRegression(columnId);
  if (type === 'logarithmic_regression') return DATASET_IDS.logarithmicRegression(columnId);
  if (type === 'exponential_regression') return DATASET_IDS.exponentialRegression(columnId);
  if (type === 'polynomial_regression') return DATASET_IDS.polynomialRegression(columnId);
  return undefined;
};

const defaultTrendlineFromDataset = (
  trendline: Trendline,
  dataTrendlineOptions: TrendlineDataset[],
  type: Trendline['type']
) => {
  let series: LineSeriesOption | undefined = undefined;
  const searchId = getSearchId(type, trendline.columnId);
  const trendlineDataset = dataTrendlineOptions.find((option) => option.id === searchId);

  if (trendlineDataset) {
    const trendlineDimensions = trendlineDataset.dimensions as string[];
    const indexOfDimension = trendlineDimensions.findIndex((dimension) => {
      const { key } = extractFieldsFromChain(dimension)[0];
      return key === trendline.columnId;
    });

    series = {
      silent: true,
      animation: false,
      name: searchId,
      type: 'line',
      z: 5,
      data: trendlineDataset.source,
      dimensions: trendlineDataset.dimensions,
      encode: { x: 0, y: indexOfDimension },
      symbolSize: 0,
      smooth: true,
      lineStyle: {
        color: trendline.trendLineColor || busterLightTheme.theme.markLine.lineStyle.color
      },
      endLabel: {
        show: trendline.showTrendlineLabel,
        formatter: () => trendline.trendlineLabel || trendlineDataset.equation!
      }
    };
  }

  return series;
};
