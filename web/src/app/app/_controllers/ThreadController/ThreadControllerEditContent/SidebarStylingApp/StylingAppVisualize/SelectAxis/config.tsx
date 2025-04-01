import { AppMaterialIcons } from '@/components/icons/AppMaterialIcons';
import { ChartEncodes, ChartType, type IColumnLabelFormat } from '@/components/charts';
import React from 'react';

export const ColumnTypeIcon: Record<
  IColumnLabelFormat['style'],
  {
    icon: React.ReactNode;
    value: IColumnLabelFormat['style'];
    tooltip: string;
  }
> = {
  string: {
    icon: <AppMaterialIcons data-value="string" icon="text_fields" />,
    value: 'string',
    tooltip: 'Text'
  },
  number: {
    icon: <AppMaterialIcons data-value="number" icon="numbers" />,
    value: 'number',
    tooltip: 'Number'
  },
  date: {
    icon: <AppMaterialIcons data-value="date" icon="calendar_today" />,
    value: 'date',
    tooltip: 'Date'
  },
  currency: {
    icon: <AppMaterialIcons data-value="currency" icon="attach_money" />,
    value: 'currency',
    tooltip: 'Currency'
  },
  percent: {
    icon: <AppMaterialIcons data-value="percent" icon="percent" />,
    value: 'percent',
    tooltip: 'Percent'
  }
};

export enum SelectAxisContainerId {
  Available = 'available',
  XAxis = 'xAxis',
  YAxis = 'yAxis',
  CategoryAxis = 'categoryAxis',
  SizeAxis = 'sizeAxis',
  Tooltip = 'tooltip',
  Y2Axis = 'y2Axis',
  Metric = 'metric'
}

export const zoneIdToAxis: Record<SelectAxisContainerId, string> = {
  [SelectAxisContainerId.Available]: '',
  [SelectAxisContainerId.XAxis]: 'x',
  [SelectAxisContainerId.YAxis]: 'y',
  [SelectAxisContainerId.CategoryAxis]: 'category',
  [SelectAxisContainerId.SizeAxis]: 'size',
  [SelectAxisContainerId.Tooltip]: 'tooltip',
  [SelectAxisContainerId.Y2Axis]: 'y2',
  [SelectAxisContainerId.Metric]: 'metric'
};

export const chartTypeToAxis: Record<
  ChartType,
  'barAndLineAxis' | 'scatterAxis' | 'pieChartAxis' | 'comboChartAxis' | ''
> = {
  [ChartType.Bar]: 'barAndLineAxis',
  [ChartType.Line]: 'barAndLineAxis',
  [ChartType.Scatter]: 'scatterAxis',
  [ChartType.Pie]: 'pieChartAxis',
  [ChartType.Combo]: 'comboChartAxis',
  [ChartType.Metric]: '',
  [ChartType.Table]: ''
};
