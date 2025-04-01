import { determineFontColorContrast, formatLabel } from '@/utils';
import type { Context } from 'chartjs-plugin-datalabels';
import { formatChartLabelDelimiter } from '../../commonHelpers';
import { extractFieldsFromChain, appendToKeyValueChain } from '../../chartHooks';
import { BusterChartProps, ColumnLabelFormat } from '../../interfaces';

export const dataLabelFontColorContrast = (context: Context) => {
  const color = context.dataset.backgroundColor as string;
  return determineFontColorContrast(color);
};

export const formatChartLabel = (
  label: string,
  columnLabelFormats: NonNullable<BusterChartProps['columnLabelFormats']>,
  hasMultipleMeasures: boolean,
  hasCategoryAxis: boolean
): string => {
  if (hasCategoryAxis && !hasMultipleMeasures) {
    const fields = extractFieldsFromChain(label);
    const lastField = fields.at(0)!;
    const newLabel = appendToKeyValueChain(lastField);
    return formatChartLabelDelimiter(newLabel, columnLabelFormats);
  }

  if (!hasMultipleMeasures) {
    const fields = extractFieldsFromChain(label);
    const lastField = fields.at(-1)!;
    return formatChartLabelDelimiter(lastField.value || lastField.key, columnLabelFormats);
  }

  return formatChartLabelDelimiter(label, columnLabelFormats);
};

export const formatBarAndLineDataLabel = (
  value: number,
  context: Context,
  usePercentage: boolean | undefined,
  columnLabelFormat: ColumnLabelFormat
) => {
  if (!usePercentage) {
    return formatLabel(value, columnLabelFormat);
  }

  const shownDatasets = context.chart.data.datasets.filter(
    (dataset) => !dataset.hidden && !dataset.isTrendline
  );
  const hasMultipleDatasets = shownDatasets.length > 1;
  const total: number = hasMultipleDatasets
    ? context.chart.$totalizer.stackTotals[context.dataIndex]
    : context.chart.$totalizer.seriesTotals[context.datasetIndex];
  const percentage = ((value as number) / total) * 100;

  return formatLabel(percentage, {
    ...columnLabelFormat,
    style: 'percent',
    columnType: 'number'
  });
};
