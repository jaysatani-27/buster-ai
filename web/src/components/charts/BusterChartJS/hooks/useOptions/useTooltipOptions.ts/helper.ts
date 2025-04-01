import { extractFieldsFromChain } from '@/components/charts/chartHooks';
import type { BusterChartProps } from '@/components/charts/interfaces';
import { formatLabel } from '@/utils';
import type { Chart } from 'chart.js';

export const getPercentage = (
  rawValue: number,
  dataIndex: number,
  datasetIndex: number,
  datasetKey: string,
  columnLabelFormats: NonNullable<BusterChartProps['columnLabelFormats']>,
  chart: Chart,
  hasMultipleShownDatasets: boolean
) => {
  if (hasMultipleShownDatasets) {
    return getStackedPercentage(rawValue, dataIndex, datasetKey, columnLabelFormats, chart);
  }

  return getSeriesPercentage(rawValue, datasetIndex, datasetKey, columnLabelFormats, chart);
};

const getSeriesPercentage = (
  rawValue: number,
  datasetIndex: number,
  key: string,
  columnLabelFormats: NonNullable<BusterChartProps['columnLabelFormats']>,
  chart: Chart
): string => {
  const total = chart.$totalizer.seriesTotals[datasetIndex];
  const percentage = (rawValue / total) * 100;
  return percentageFormatter(percentage, key, columnLabelFormats);
};

const getStackedPercentage = (
  rawValue: number,
  dataPointIndex: number,
  key: string,
  columnLabelFormats: NonNullable<BusterChartProps['columnLabelFormats']>,
  chart: Chart
) => {
  const stackTotal = chart.$totalizer.stackTotals[dataPointIndex!];
  const percentage = (rawValue / (stackTotal || 1)) * 100;
  return percentageFormatter(percentage, key, columnLabelFormats);
};

export const percentageFormatter = (
  percentage: number,
  axisKeyUnDeliminated: string,
  columnLabelFormats: NonNullable<BusterChartProps['columnLabelFormats']>
) => {
  const { key } = extractFieldsFromChain(axisKeyUnDeliminated)[0];
  let columnLabelFormat = columnLabelFormats[key];
  const isPercentage = columnLabelFormat?.style === 'percent';
  if (!isPercentage) {
    columnLabelFormat = {
      style: 'percent',
      columnType: 'number'
    };
  }
  return formatLabel(percentage, columnLabelFormat, false);
};
