import { AppMaterialIcons } from '@/components/icons';
import { ViewType, ChartType } from './interfaces';

export const viewTypeOptions = [
  {
    label: 'Chart',
    value: ViewType.Chart,
    icon: <AppMaterialIcons icon="monitoring" />
  },
  {
    label: 'Table',
    value: ViewType.Table,
    icon: <AppMaterialIcons icon="table" />
  }
];

export const chartOptions = [
  {
    label: 'Bar chart',
    value: ChartType.Bar,
    icon: <AppMaterialIcons icon="bar_chart"></AppMaterialIcons>
  },
  {
    label: 'Line chart',
    value: ChartType.Line,
    icon: <AppMaterialIcons icon="stacked_line_chart"></AppMaterialIcons>
  },
  {
    label: 'Pie chart',
    value: ChartType.Pie,
    icon: <AppMaterialIcons icon="pie_chart"></AppMaterialIcons>
  },
  {
    label: 'Scatter chart',
    value: ChartType.Scatter,
    icon: <AppMaterialIcons icon="bubble_chart"></AppMaterialIcons>
  },
  {
    label: 'Metric chart',
    value: ChartType.Metric,
    icon: <AppMaterialIcons icon="looks_one"></AppMaterialIcons>
  }
];
