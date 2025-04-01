import type { BusterChartConfigProps, ChartType, IColumnLabelFormat } from '../interfaces';

export type BusterTableChartConfig = {
  type: ChartType.Table;
  tableColumnOrder?: BusterChartConfigProps['tableColumnOrder'];
  tableColumnWidths?: BusterChartConfigProps['tableColumnWidths'];
  tableHeaderBackgroundColor?: string | null;
  tableHeaderFontColor?: string | null;
  tableColumnFontColor?: string | null;
  columnLabelFormats?: Record<string, IColumnLabelFormat>;
};
