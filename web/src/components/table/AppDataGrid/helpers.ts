import set from 'lodash/set';
import clamp from 'lodash/clamp';
import { makeHumanReadble } from '@/utils/text';

export const MAX_WIDTH = 950;
export const MIN_WIDTH = 100;

export const defaultHeaderFormat = (v: any) => makeHumanReadble(v);
export const defaultCellFormat = (v: any) => v;

export const createInitialColumnWidths = (
  allUniqueFields: string[],
  sampleOfRows: Record<string, string | number | Date | null>[],
  headerFormat: (value: any, columnName: string) => string,
  cellFormat: (value: any, columnName: string) => string,
  columnWidths?: Record<string, number>,
  widthOfContainer?: number | undefined,
  columnOrder?: string[]
) => {
  const sortedUniqueFields = columnOrder
    ? allUniqueFields.sort((a, b) => columnOrder.indexOf(a) - columnOrder.indexOf(b))
    : allUniqueFields;

  let fields: Record<string, number> = {};
  sortedUniqueFields.map((field, index) => {
    const width = columnWidths?.[field] || 0;
    const isLast = index === sortedUniqueFields.length - 1;
    if (isLast && widthOfContainer) {
      const widthOfAllColumns = Object.values(fields).reduce((acc, curr) => acc + curr, 0);
      if (widthOfAllColumns + width < widthOfContainer) {
        const fillInWidth = widthOfContainer - widthOfAllColumns;
        fields = set(fields, field, fillInWidth);
        return;
      }
    }

    if (width) {
      fields = set(fields, field, width);
      return;
    }

    const samplesFromData = sampleOfRows.map((row) => {
      const value = row[field];
      return cellFormat(value, field);
    });
    const title = headerFormat(field, field);
    const longestValue = [...samplesFromData, title].reduce<string>((acc, curr) => {
      if (String(curr).length > String(acc).length) {
        return String(curr);
      }
      return acc;
    }, title || '');
    const maxWidth = clamp(String(longestValue).length * 10, 50, MAX_WIDTH);
    const minWidth = clamp(String(longestValue).length * 10, MIN_WIDTH, MAX_WIDTH);
    const isUnderMaxWidth = maxWidth < MAX_WIDTH;
    const maxWidthValue = isUnderMaxWidth ? maxWidth : maxWidth;
    const maxWidthValueDiv = isUnderMaxWidth && !!maxWidthValue ? maxWidth : maxWidthValue!;
    fields = set(fields, field, maxWidthValueDiv);
  });

  return fields;
};
