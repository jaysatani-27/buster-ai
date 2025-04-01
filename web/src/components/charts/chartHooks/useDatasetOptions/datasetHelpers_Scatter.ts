import { BusterChartProps, ColumnLabelFormat } from '../../interfaces';
import { ScatterAxis } from '../../interfaces/axisInterfaces';
import { createDimension } from './datasetHelpers_BarLinePie';
import { appendToKeyValueChain } from './groupingHelpers';
import { DatasetOption } from './interfaces';

type DataItem = NonNullable<BusterChartProps['data']>[number];

export const mapScatterData = (
  sortedData: NonNullable<BusterChartProps['data']>,
  categoryFields: string[]
) => {
  const xValuesSet = new Set<string>(); // Stores unique X-axis values
  const categoriesSet = new Set<string>(); // Stores unique category values
  const dataMap = new Map<string | number, Record<string, string | number | Date>>(); // Stores data points in format: "xValue|category" => measures

  const categoryFieldAccessors = categoryFields.map((field) => (item: DataItem) => ({
    key: field,
    value: String(item[field])
  }));

  sortedData.forEach((item) => {
    const categoryKey = appendToKeyValueChain(
      categoryFieldAccessors.map((accessor) => accessor(item))
    );
    categoriesSet.add(categoryKey);
  });

  return { dataMap, xValuesSet, categoriesSet };
};

export const processScatterData = (
  data: NonNullable<BusterChartProps['data']>,
  xAxisField: string,
  measureFields: string[],
  categoryFields: string[],
  sizeFieldArray: ScatterAxis['size'],
  columnLabelFormats: Record<string, ColumnLabelFormat>,
  categoriesSet: Set<string>
): (string | number | null)[][] => {
  const processedData: (string | number | null)[][] = [];
  const sizeField = sizeFieldArray?.[0];
  const categories = Array.from(categoriesSet);

  const categoryFieldAccessors = categoryFields.map((field) => (item: DataItem) => ({
    key: field,
    value: String(item[field])
  }));
  const defaultReplaceMissingDataWith = null; //null is the default for scatter charts

  data.forEach((item) => {
    const row: (string | number | null)[] = [];
    row.push(item[xAxisField] as string | number);

    const categoryKey = appendToKeyValueChain(
      categoryFieldAccessors.map((accessor) => accessor(item))
    );

    measureFields.forEach((measure) => {
      categories.forEach((category) => {
        const columnLabelFormat = columnLabelFormats[measure];
        const replaceMissingDataWith =
          columnLabelFormat?.replaceMissingDataWith !== undefined
            ? columnLabelFormat?.replaceMissingDataWith
            : defaultReplaceMissingDataWith;
        if (categoryKey === category) {
          const value = item[measure] || replaceMissingDataWith;
          row.push(value as string | number);
        } else {
          row.push(replaceMissingDataWith);
        }
      });
    });

    if (categoryFields.length > 0) {
      const categoryKey = appendToKeyValueChain(
        categoryFieldAccessors.map((accessor) => accessor(item))
      );
      row.push(categoryKey);
    }

    if (sizeField) {
      row.push(item[sizeField] as string | number);
    }

    processedData.push(row);
  });

  return processedData;
};

export const getScatterDimensions = (
  categoryAxis: Set<string>,
  xAxisField: string,
  measureFields: string[],
  sizeField: ScatterAxis['size'] = []
): string[] => {
  const categories = Array.from(categoryAxis);
  const xField = appendToKeyValueChain({ key: xAxisField, value: '' });
  const dimensions = [xField, ...createDimension(measureFields, categories)];
  if (sizeField && sizeField.length) {
    const sizeFieldDimension = appendToKeyValueChain({ key: sizeField[0], value: '' });
    dimensions.push(sizeFieldDimension);
  }
  return dimensions;
};

export const getScatterTooltipKeys = (
  tooltipFields: string[],
  xAxisField: string,
  categoriesSet: Set<string>,
  measureFields: string[],
  sizeFieldArray: ScatterAxis['size']
) => {
  const categories = Array.from(categoriesSet);
  const hasTooltipFields = tooltipFields.length > 0;
  const fieldsToUse = hasTooltipFields ? tooltipFields : measureFields;
  if (!hasTooltipFields) {
    fieldsToUse.push(xAxisField);
  }
  const sizeField = sizeFieldArray?.[0];
  if (sizeField) fieldsToUse.push(sizeField);
  return createDimension(fieldsToUse, categories);
};

export const getScatterDatasetOptions = (
  processedData: (string | number | Date | null)[][],
  dimensions: string[]
): DatasetOption[] => {
  return [
    {
      id: 'scatter-dataset',
      dimensions,
      source: processedData
    }
  ];
};
