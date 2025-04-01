import { Plugin, ChartType, ChartData, DefaultDataPoint } from 'chart.js';

type PluginDataPoint = ChartData['datasets'][0]['data'];

export type ExtendedPlugin = Plugin<ChartType, PluginOptions>;

export interface PluginOptions {
  enable: boolean;
  replaceTooltipLabel?: boolean;
  fixNegativeScale?: boolean;
  individual?: boolean;
  precision?: number;
  axisId?: string;
}

declare module 'chart.js' {
  interface ChartData<
    TType extends ChartType = ChartType,
    TData = DefaultDataPoint<TType>,
    TLabel = unknown
  > {
    calculatedData?: number[][];
    originalData?: PluginDataPoint[];
  }

  export interface PluginOptionsByType<TType extends ChartType> {
    stacked100?: PluginOptions;
  }
}

export type ExtendedChartData = ChartData;
