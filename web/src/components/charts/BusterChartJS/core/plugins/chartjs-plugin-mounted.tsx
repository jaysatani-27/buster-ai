import { ChartType, Chart, Plugin } from 'chart.js';

export interface ChartMountedPluginOptions {
  onMounted?: (chart: Chart) => void;
  onInitialAnimationEnd?: (chart: Chart) => void;
}

declare module 'chart.js' {
  interface PluginOptionsByType<TType extends ChartType> {
    chartMounted?: ChartMountedPluginOptions;
  }
}

export const ChartMountedPlugin: Plugin<ChartType, ChartMountedPluginOptions> = {
  id: 'chartMounted',
  afterInit: (chart, args, options) => {
    options?.onMounted?.(chart);
  },
  afterRender: (chart, args, options) => {
    const hasLabels = !!chart.data?.labels?.length;
    if (hasLabels) {
      options?.onInitialAnimationEnd?.(chart);
    }
  },

  defaults: {
    onMounted: () => {},
    onInitialAnimationEnd: () => {}
  }
};
