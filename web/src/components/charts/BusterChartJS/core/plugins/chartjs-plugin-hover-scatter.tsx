import { ChartType, Chart, Plugin } from 'chart.js';

export interface ChartHoverScatterPluginOptions {
  color?: string;
  lineWidth?: number;
  lineDash?: number[];
}

declare module 'chart.js' {
  interface PluginOptionsByType<TType extends ChartType> {
    hoverScatter?: ChartHoverScatterPluginOptions | false;
  }
}

export const ChartHoverScatterPlugin: Plugin<ChartType, ChartHoverScatterPluginOptions> = {
  id: 'tooltipHoverScatter',

  defaults: {
    color: 'rgba(0,0,0,0.6)',
    lineWidth: 0.65,
    lineDash: [3, 3]
  },

  beforeDraw: (chart, args, options) => {
    const { ctx, chartArea } = chart;

    // Only show crosshair for scatter and bubble charts
    const type = (chart.config as any).type as ChartType;
    if (type !== 'scatter' && type !== 'bubble') return;

    // Get mouse position from chart
    const activeElements = chart.getActiveElements();
    if (activeElements.length === 0) return;

    const activePoint = activeElements[0];
    const { x, y } = activePoint.element;

    // Draw crosshair
    ctx.save();
    ctx.beginPath();
    ctx.lineWidth = options.lineWidth || 1;
    ctx.strokeStyle = options.color || '#666';
    ctx.setLineDash(options.lineDash || [5, 5]);

    // Vertical line
    ctx.moveTo(x, chartArea.top);
    ctx.lineTo(x, chartArea.bottom);

    // Horizontal line
    ctx.moveTo(chartArea.left, y);
    ctx.lineTo(chartArea.right, y);

    ctx.stroke();
    ctx.restore();
  }
};
