import type { DatasetOption, TrendlineDataset } from '@/components/charts/chartHooks';
import type {
  BusterChartProps,
  ChartEncodes,
  IColumnLabelFormat,
  ScatterAxis,
  Trendline
} from '@/components/charts/interfaces';

export interface SeriesBuilderProps {
  selectedDataset: DatasetOption;
  allYAxisKeysIndexes: {
    name: string;
    index: number;
  }[];
  allY2AxisKeysIndexes: {
    name: string;
    index: number;
  }[];
  columnSettings: NonNullable<BusterChartProps['columnSettings']>;
  colors: string[];
  columnLabelFormats: Record<string, IColumnLabelFormat>;
  xAxisKeys: ChartEncodes['x'];
  categoryKeys: ScatterAxis['category'];
  sizeKeyIndex: {
    name: string;
    index: number;
    minValue: number;
    maxValue: number;
  } | null;
  scatterDotSize: BusterChartProps['scatterDotSize'];
  lineGroupType: BusterChartProps['lineGroupType'];
  selectedChartType: BusterChartProps['selectedChartType'];
  barShowTotalAtTop: BusterChartProps['barShowTotalAtTop'];
}
