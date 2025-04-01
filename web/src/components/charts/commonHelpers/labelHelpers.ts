import { formatLabel } from '@/utils';
import isEmpty from 'lodash/isEmpty';
import type { ColumnLabelFormat } from '../interfaces';
import { extractFieldsFromChain } from '../chartHooks';

export const JOIN_CHARACTER = ' | ';

export const formatChartLabelDelimiter = (
  text: string,
  columnLabelFormats: Record<string, ColumnLabelFormat>
): string => {
  let fields = extractFieldsFromChain(text);

  if (!fields || fields.length === 0) {
    return ''; //I used to return text? Maybe I should?
  }

  const formattedFields = fields.map((field) => formatLabelField(field, columnLabelFormats));

  return formattedFields.join(JOIN_CHARACTER);
};

//used in the legend and axis labels. exported only for tooltip
const formatLabelField = (
  field: { value: string; key: string },
  columnLabelFormats: Record<string, ColumnLabelFormat>
) => {
  const { value, key = '' } = field;
  const hasValue = !isEmpty(value) && typeof value !== 'number';
  const columnLabelFormat = columnLabelFormats[key];
  return formatLabel(hasValue ? value : key, columnLabelFormat, !hasValue);
};

export const formatChartValueDelimiter = (
  rawValue: string | number,
  columnNameDelimiter: string,
  columnLabelFormats: Record<string, ColumnLabelFormat>
) => {
  const fields = extractFieldsFromChain(columnNameDelimiter);
  const lastField = fields[fields.length - 1]; //if there are categories, the last field is the value
  const { key: columnName } = lastField;
  return formatValueField(rawValue, columnName, columnLabelFormats);
};

const formatValueField = (
  rawValue: string | number,
  columnName: string = '', //must be columnName, not delimiter
  columnLabelFormats: Record<string, ColumnLabelFormat>
) => {
  return formatLabel(rawValue, columnLabelFormats[columnName], false);
};
