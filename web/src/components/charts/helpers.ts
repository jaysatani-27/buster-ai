import { BarAndLineAxis, ChartEncodes } from './interfaces';
import { ChartType, ViewType } from './interfaces/enum';
import isEmpty from 'lodash/isEmpty';

const defaultAxisCheck = (selectedAxis: ChartEncodes) => {
  if (isEmpty(selectedAxis.x) || isEmpty(selectedAxis.y)) return false;
  return true;
};

const AxisMethodCheckRecord: Record<ChartType, (selectedAxis: ChartEncodes) => boolean> = {
  [ChartType.Line]: defaultAxisCheck,
  [ChartType.Bar]: defaultAxisCheck,
  [ChartType.Scatter]: defaultAxisCheck,
  [ChartType.Pie]: defaultAxisCheck,
  [ChartType.Combo]: defaultAxisCheck,
  [ChartType.Metric]: () => true,
  [ChartType.Table]: () => true
};

export const doesChartHaveValidAxis = ({
  selectedChartType,
  selectedAxis,
  isTable
}: {
  selectedChartType: ChartType;
  selectedAxis: ChartEncodes | undefined;
  isTable: boolean;
}) => {
  if (isTable) return true;
  return AxisMethodCheckRecord[selectedChartType](selectedAxis!);
};
