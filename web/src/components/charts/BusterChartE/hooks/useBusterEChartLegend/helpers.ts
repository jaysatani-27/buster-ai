import type {
  DatasetComponentOption,
  LineSeriesOption,
  PieSeriesOption,
  SeriesOption
} from 'echarts';
import { BusterChartLegendItem } from '../../../BusterChartLegend';
import { BusterChartConfigProps, BusterChartProps, ChartType } from '../../../interfaces';
import { TOTAL_INNER_LABEL_ID } from '../useSeriesOptions/PieChartSeriesBuilder';
import { formatChartLabelDelimiter, JOIN_CHARACTER } from '../../../commonHelpers';

export const getLegendItems = ({
  colors,
  datasets,
  series,
  columnLabelFormats,
  inactiveDatasets,
  selectedChartType,
  allYAxisColumnNames
}: {
  colors: string[];
  datasets: DatasetComponentOption[];
  series: (SeriesOption | PieSeriesOption)[];
  allYAxisColumnNames: string[];
  columnLabelFormats: NonNullable<BusterChartConfigProps['columnLabelFormats']>;
  inactiveDatasets: Record<string, boolean>;
  selectedChartType: NonNullable<BusterChartProps['selectedChartType']>;
}): BusterChartLegendItem[] => {
  /*
   * THE SERIES NAME IS RAW AND UNFORMATTED
   */

  //PIE CHART is the exception because we need to map each row to a legend item
  if (selectedChartType === 'pie') {
    const dimensionIndex = 0; //The title is always the first dimension
    const hasMultipleYAxis = allYAxisColumnNames.length > 1;

    const formatPieLabelName = (name: string, serieName: string) => {
      if (hasMultipleYAxis) {
        const formattedSeriesName = formatChartLabelDelimiter(serieName, columnLabelFormats);
        const formattedName = formatChartLabelDelimiter(name, columnLabelFormats);
        return [formattedName, formattedSeriesName].join(JOIN_CHARACTER);
      }

      return formatChartLabelDelimiter(name, columnLabelFormats);
    };

    return series.flatMap<BusterChartLegendItem>((serie, serieIndex) => {
      if (serie.name === TOTAL_INNER_LABEL_ID) {
        return [];
      }
      const { datasetIndex = 0 } = serie as PieSeriesOption;
      const assosciatedDataset = datasets[datasetIndex];
      const assosciatedDatasetSource = assosciatedDataset.source as Record<
        string,
        string | number
      >[];

      return assosciatedDatasetSource?.map<BusterChartLegendItem>((row, rowIndex) => {
        const id = row[dimensionIndex] as string;
        return {
          type: ChartType.Pie,
          color: colors[rowIndex % colors.length],
          //we need to use the id + serieName to check if the dataset is active
          inactive: inactiveDatasets[id + serie.name] ?? false,
          id,
          serieName: serie.name as string,
          formattedName: formatPieLabelName(row[dimensionIndex] as string, serie.name as string)
        };
      });
    });
  }

  const isComboChart = selectedChartType === ChartType.Combo;

  //DEFAULT IS FOR LINE AND BAR CHARTS
  return series.map<BusterChartLegendItem>((series, index) => {
    const formattedName = formatChartLabelDelimiter(series.name as string, columnLabelFormats);

    return {
      formattedName,
      color: colors[index % colors.length],
      inactive: inactiveDatasets[series.name as string] ?? false,
      type: typeSelector(series.type as SeriesOption['type'], series, isComboChart),
      id: series.name as string
    };
  });
};

const typeSelector = (
  type: SeriesOption['type'],
  series: SeriesOption,
  isComboChart: boolean
): ChartType => {
  if (isComboChart && type === 'line') {
    const lineSeries = series as LineSeriesOption;
    const lineIsTransparent = lineSeries.lineStyle?.color === 'transparent';

    if (lineIsTransparent) {
      return ChartType.Scatter;
    }

    return ChartType.Line;
  }

  return ChartType.Bar;
};
