import type { SeriesOption } from 'echarts';
import { ChartType } from '../../../interfaces';

export const ChartTypeToSeriesType: Record<ChartType, SeriesOption['type']> = {
  [ChartType.Bar]: 'bar',
  [ChartType.Line]: 'line',
  [ChartType.Pie]: 'pie',
  [ChartType.Scatter]: 'scatter',
  [ChartType.Metric]: 'gauge',
  [ChartType.Table]: 'bar', //Won't actually be used
  [ChartType.Combo]: 'line' //Won't actually be used
};
