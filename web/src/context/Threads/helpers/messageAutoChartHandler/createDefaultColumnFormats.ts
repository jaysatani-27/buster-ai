import {
  DEFAULT_COLUMN_LABEL_FORMAT,
  type ColumnMetaData,
  type IBusterThreadMessageChartConfig
} from '@/api/buster_rest/threads';
import type { ColumnLabelFormat, IColumnLabelFormat } from '@/components/charts';
import {
  isDateColumnType,
  isNumericColumnType,
  simplifyColumnType,
  type SimplifiedColumnType
} from '@/utils/messages';

export const createDefaultColumnLabelFormats = (
  columnLabelFormats: Record<string, IColumnLabelFormat> | undefined,
  columnsMetaData: ColumnMetaData[] | undefined
): IBusterThreadMessageChartConfig['columnLabelFormats'] => {
  if (!columnsMetaData) return {};

  return columnsMetaData.reduce(
    (acc, column) => {
      const existingLabelFormat = columnLabelFormats?.[column.name] || {};
      acc[column.name] = {
        ...createDefaulColumnLabel(columnsMetaData, column.name),
        ...existingLabelFormat
      };
      return acc;
    },
    {} as IBusterThreadMessageChartConfig['columnLabelFormats']
  );
};

const createDefaultReplaceMissingDataWith = (simpleType: SimplifiedColumnType) => {
  if (simpleType === 'number') return 0;
  if (simpleType === 'string') return null;
  if (simpleType === 'date') return null;
  return null;
};

const createDefaulColumnLabel = (
  columnsMetaData: ColumnMetaData[],
  name: string
): Required<ColumnLabelFormat> => {
  const assosciatedColumn = columnsMetaData?.find((m) => m.name === name)!;
  const columnType: SimplifiedColumnType = simplifyColumnType(assosciatedColumn?.simple_type);
  const style = createDefaultColumnLabelStyle(columnType);
  const replaceMissingDataWith = createDefaultReplaceMissingDataWith(columnType);

  return {
    ...DEFAULT_COLUMN_LABEL_FORMAT,
    style,
    columnType,
    replaceMissingDataWith
  };
};

const createDefaultColumnLabelStyle = (
  columnType: SimplifiedColumnType
): IColumnLabelFormat['style'] => {
  if (isDateColumnType(columnType)) return 'date';
  if (isNumericColumnType(columnType)) return 'number';
  return 'string';
};
