import type { ChartProps } from '../../core';
import { LabelBuilderProps } from './useSeriesOptions';
import { formatChartLabelDelimiter } from '../../../commonHelpers';
import { SeriesBuilderProps } from './interfaces';

type PieSerieType =
  | ChartProps<'pie'>['data']['datasets'][number]
  | ChartProps<'doughnut'>['data']['datasets'][number];

export const pieSeriesBuilder_data = ({
  selectedDataset,
  allYAxisKeysIndexes,
  colors
}: SeriesBuilderProps): PieSerieType[] => {
  return allYAxisKeysIndexes.map<PieSerieType>((yAxisItem) => {
    return {
      label: yAxisItem.name,
      backgroundColor: colors,
      //pie will only have one dataset
      data: selectedDataset.source.map((item) => item[yAxisItem.index] as number),
      borderColor: 'white' //I tried to set this globally in the theme but it didn't work
    };
  });
};

export const pieSeriesBuilder_labels = (props: LabelBuilderProps) => {
  const { dataset, columnLabelFormats } = props;
  return dataset.source.map<string>((item) => {
    return formatChartLabelDelimiter(item[0] as string, columnLabelFormats);
  });
};
