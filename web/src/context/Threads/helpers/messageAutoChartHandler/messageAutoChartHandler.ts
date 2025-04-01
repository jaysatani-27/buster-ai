import {
  type BusterThreadMessage,
  type IBusterThreadMessageChartConfig,
  DEFAULT_CHART_CONFIG,
  DEFAULT_CHART_CONFIG_ENTRIES
} from '@/api/buster_rest/threads';
import type { BusterChartConfigProps } from '@/components/charts';
import { produce } from 'immer';
import isEmpty from 'lodash/isEmpty';
import { createDefaultColumnLabelFormats } from './createDefaultColumnFormats';
import { createDefaultColumnSettings } from './createDefaultColumnSettings';
import {
  createDefaultBarAndLineAxis,
  createDefaultPieAxis,
  createDefaultScatterAxis
} from './createDefaultAxis';

const keySpecificHandlers: Partial<
  Record<
    keyof IBusterThreadMessageChartConfig,
    (
      value: any,
      dataMetadata: BusterThreadMessage['data_metadata'] | undefined,
      pieChartAxis: IBusterThreadMessageChartConfig['pieChartAxis'] | undefined
    ) => any
  >
> = {
  colors: (colors: IBusterThreadMessageChartConfig['colors']) => {
    if (isEmpty(colors)) return DEFAULT_CHART_CONFIG.colors;
    if (colors.length >= 3) return colors; //we need at least 3 colors for the chart icons
    return Array.from({ length: 3 }, (_, index) => colors[index % colors.length]);
  },
  scatterDotSize: (scatterDotSize: IBusterThreadMessageChartConfig['scatterDotSize']) => {
    if (isEmpty(scatterDotSize)) return DEFAULT_CHART_CONFIG.scatterDotSize;
    return scatterDotSize;
  },
  barAndLineAxis: (
    barAndLineAxis: IBusterThreadMessageChartConfig['barAndLineAxis'],
    dataMetadata
  ) => {
    if (isEmpty(barAndLineAxis)) {
      return createDefaultBarAndLineAxis(dataMetadata?.column_metadata);
    }
    return {
      x: barAndLineAxis.x || DEFAULT_CHART_CONFIG.barAndLineAxis.x,
      y: barAndLineAxis.y || DEFAULT_CHART_CONFIG.barAndLineAxis.y,
      tooltip: barAndLineAxis.tooltip || DEFAULT_CHART_CONFIG.barAndLineAxis.tooltip,
      category: barAndLineAxis.category || DEFAULT_CHART_CONFIG.barAndLineAxis.category
    };
  },
  pieChartAxis: (pieChartAxis: IBusterThreadMessageChartConfig['pieChartAxis'], dataMetadata) => {
    if (isEmpty(pieChartAxis)) return createDefaultPieAxis(dataMetadata?.column_metadata);
    return {
      x: pieChartAxis.x || DEFAULT_CHART_CONFIG.pieChartAxis.x,
      y: pieChartAxis.y || DEFAULT_CHART_CONFIG.pieChartAxis.y,
      tooltip: pieChartAxis.tooltip || DEFAULT_CHART_CONFIG.pieChartAxis.tooltip
    };
  },
  scatterAxis: (scatterAxis: IBusterThreadMessageChartConfig['scatterAxis'], dataMetadata) => {
    if (isEmpty(scatterAxis)) return createDefaultScatterAxis(dataMetadata?.column_metadata);
    return {
      x: scatterAxis.x || DEFAULT_CHART_CONFIG.scatterAxis.x,
      y: scatterAxis.y || DEFAULT_CHART_CONFIG.scatterAxis.y,
      size: scatterAxis.size || DEFAULT_CHART_CONFIG.scatterAxis.size,
      tooltip: scatterAxis.tooltip || DEFAULT_CHART_CONFIG.scatterAxis.tooltip,
      category: scatterAxis.category || DEFAULT_CHART_CONFIG.scatterAxis.category
    };
  },
  comboChartAxis: (
    comboChartAxis: IBusterThreadMessageChartConfig['comboChartAxis'],
    dataMetadata
  ) => {
    if (isEmpty(comboChartAxis)) return createDefaultBarAndLineAxis(dataMetadata?.column_metadata);
    return {
      x: comboChartAxis.x || DEFAULT_CHART_CONFIG.comboChartAxis.x,
      y: comboChartAxis.y || DEFAULT_CHART_CONFIG.comboChartAxis.y,
      y2: comboChartAxis.y2 || DEFAULT_CHART_CONFIG.comboChartAxis.y2,
      tooltip: comboChartAxis.tooltip || DEFAULT_CHART_CONFIG.comboChartAxis.tooltip,
      category: comboChartAxis.category || DEFAULT_CHART_CONFIG.comboChartAxis.category
    };
  },
  metricColumnId: (
    metricColumnId: IBusterThreadMessageChartConfig['metricColumnId'],
    dataMetadata
  ) => {
    if (isEmpty(metricColumnId)) {
      const firstNumberColumn = dataMetadata?.column_metadata?.find(
        (m) => m.simple_type === 'number'
      );
      return firstNumberColumn?.name || dataMetadata?.column_metadata?.[0]?.name || '';
    }
    return metricColumnId;
  },
  metricHeader: (metricHeader: IBusterThreadMessageChartConfig['metricHeader']) => {
    if (isEmpty(metricHeader)) return DEFAULT_CHART_CONFIG.metricHeader;
    return metricHeader;
  },
  metricSubHeader: (metricSubHeader: IBusterThreadMessageChartConfig['metricSubHeader']) => {
    if (isEmpty(metricSubHeader)) return DEFAULT_CHART_CONFIG.metricSubHeader;
    return metricSubHeader;
  },
  columnLabelFormats: (
    columnLabelFormats: IBusterThreadMessageChartConfig['columnLabelFormats'],
    dataMetadata
  ) => {
    return createDefaultColumnLabelFormats(columnLabelFormats, dataMetadata?.column_metadata);
  },
  columnSettings: (
    columnSettings: IBusterThreadMessageChartConfig['columnSettings'],
    dataMetadata
  ) => {
    return createDefaultColumnSettings(columnSettings, dataMetadata?.column_metadata);
  },
  pieLabelPosition: (
    pieLabelPosition: IBusterThreadMessageChartConfig['pieLabelPosition'],
    dataMetadata,
    pieChartAxis
  ) => {
    // if (isEmpty(pieLabelPosition)) {
    //   const firstPieColumn = pieChartAxis?.x?.[0];
    //   const firstPieColumnMetaData = dataMetadata?.column_metadata?.find(
    //     ({ name }) => name === firstPieColumn
    //   );
    //   const hasMoreThanXRows = (firstPieColumnMetaData?.unique_values || 0) > 6;
    //   return !hasMoreThanXRows ? 'none' : 'none';
    // }
    return pieLabelPosition;
  }
};

export const createDefaultChartConfig = (
  message: Pick<BusterThreadMessage, 'chart_config' | 'data_metadata'>
): IBusterThreadMessageChartConfig => {
  const chartConfig: BusterChartConfigProps | undefined = message.chart_config;
  const dataMetadata = message.data_metadata;
  const pieChartAxis = chartConfig?.pieChartAxis;

  const newChartConfig = produce(DEFAULT_CHART_CONFIG, (draft) => {
    DEFAULT_CHART_CONFIG_ENTRIES.forEach(([key, defaultValue]) => {
      const _key = key as keyof IBusterThreadMessageChartConfig;
      const chartConfigValue = chartConfig?.[_key];

      const handler = keySpecificHandlers[_key];

      if (!handler) {
        (draft as any)[_key] = chartConfigValue ?? defaultValue;
        return;
      }

      const result = handler(chartConfigValue, dataMetadata, pieChartAxis);

      (draft as any)[_key] = result ?? defaultValue;
    });
  });

  return newChartConfig;
};
