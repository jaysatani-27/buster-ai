import {
  DEFAULT_COLUMN_SETTINGS,
  type ColumnMetaData,
  type IBusterThreadMessageChartConfig
} from '@/api/buster_rest/threads';
import type { ColumnSettings } from '@/components/charts';

export const createDefaultColumnSettings = (
  existingColumnSettings: Record<string, ColumnSettings> | undefined,
  columnsMetaData: ColumnMetaData[] | undefined
): IBusterThreadMessageChartConfig['columnSettings'] => {
  if (!columnsMetaData) return {};

  return columnsMetaData.reduce<IBusterThreadMessageChartConfig['columnSettings']>(
    (acc, column) => {
      acc[column.name] = {
        ...DEFAULT_COLUMN_SETTINGS,
        ...(existingColumnSettings?.[column.name] || {})
      };
      return acc;
    },
    {}
  );
};
