import { extractFieldsFromChain } from '@/components/charts/chartHooks';
import type { ChartProps } from '../../core';
import { SeriesBuilderProps } from './interfaces';
import { LabelBuilderProps } from './useSeriesOptions';
import { formatChartLabelDelimiter, formatYAxisLabel, yAxisSimilar } from '../../../commonHelpers';
import { dataLabelFontColorContrast, formatBarAndLineDataLabel } from '../../helpers';
import { BarElement } from 'chart.js';
import { Context } from 'chartjs-plugin-datalabels';
import { defaultLabelOptionConfig } from '../useChartSpecificOptions/labelOptionConfig';
import type { Options } from 'chartjs-plugin-datalabels/types/options';
import { DEFAULT_CHART_LAYOUT } from '../../ChartJSTheme';

export const barSeriesBuilder = ({
  selectedDataset,
  allYAxisKeysIndexes,
  colors,
  columnSettings,
  columnLabelFormats,
  xAxisKeys,
  barShowTotalAtTop
}: SeriesBuilderProps): ChartProps<'bar'>['data']['datasets'] => {
  const dataLabelOptions: Options['labels'] = {};

  if (barShowTotalAtTop) {
    const yAxis = allYAxisKeysIndexes.map((yAxis) => {
      const key = extractFieldsFromChain(yAxis.name).at(-1)!?.key;
      return key;
    });

    let hasBeenDrawn = false;

    dataLabelOptions.stackTotal = {
      display: function (context) {
        const shownDatasets = context.chart.data.datasets.filter(
          (dataset) => !dataset.hidden && !dataset.isTrendline
        );

        const canDisplay = context.datasetIndex === shownDatasets.length - 1;

        if (canDisplay && !hasBeenDrawn) {
          const chartLayout = context.chart.options.layout;
          const padding = { ...DEFAULT_CHART_LAYOUT.padding, top: 24 };
          context.chart.options.layout = { ...chartLayout, padding };
          requestAnimationFrame(() => {
            context.chart.update(); //this is hack because the chart data label almost always overflows
          });
          hasBeenDrawn = true;
        }

        return canDisplay;
      },
      formatter: function (_, context) {
        const canUseSameYFormatter = yAxisSimilar(yAxis, columnLabelFormats);
        const value = context.chart.$totalizer.stackTotals[context.dataIndex];
        return formatYAxisLabel(
          value,
          yAxis,
          canUseSameYFormatter,
          columnLabelFormats,
          false,
          false
        );
      },
      anchor: 'end',
      align: 'end',
      ...defaultLabelOptionConfig
    } as NonNullable<Options['labels']>['stackTotal'];
  }

  return allYAxisKeysIndexes.map<ChartProps<'bar'>['data']['datasets'][number]>(
    (yAxisItem, index) => {
      return barBuilder({
        selectedDataset,
        colors,
        columnSettings,
        columnLabelFormats,
        yAxisItem,
        index,
        xAxisKeys,
        dataLabelOptions
      });
    }
  );
};

declare module 'chart.js' {
  interface Chart {
    $barDataLabels: Record<
      number,
      Record<
        number,
        {
          formattedValue: string;
          rotation: number;
        }
      >
    >;
  }
}

export const barBuilder = ({
  selectedDataset,
  colors,
  columnSettings,
  columnLabelFormats,
  yAxisItem,
  index,
  yAxisID,
  order,
  xAxisKeys,
  dataLabelOptions
}: Pick<
  SeriesBuilderProps,
  'selectedDataset' | 'colors' | 'columnSettings' | 'columnLabelFormats'
> & {
  yAxisItem: SeriesBuilderProps['allYAxisKeysIndexes'][number];
  index: number;
  yAxisID?: string;
  order?: number;
  xAxisKeys: string[];
  dataLabelOptions?: Options['labels'];
}): ChartProps<'bar'>['data']['datasets'][number] => {
  const yKey = extractFieldsFromChain(yAxisItem.name).at(-1)!?.key;
  const columnSetting = columnSettings[yKey];
  const columnLabelFormat = columnLabelFormats[yKey];
  const usePercentage = !!columnSetting?.showDataLabelsAsPercentage;
  const showLabels = !!columnSetting?.showDataLabels;

  const setBarDataLabelsManager = (context: Context, formattedValue: string, rotation: number) => {
    const dataIndex = context.dataIndex;
    const datasetIndex = context.datasetIndex;
    context.chart.$barDataLabels = {
      ...context.chart.$barDataLabels,
      [datasetIndex]: {
        ...context.chart.$barDataLabels?.[datasetIndex],
        [dataIndex]: {
          formattedValue,
          rotation
        }
      }
    };
  };

  const getBarDataLabelsManager = (context: Context) => {
    const dataIndex = context.dataIndex;
    const datasetIndex = context.datasetIndex;
    const values = context.chart.$barDataLabels?.[datasetIndex]?.[dataIndex];

    return {
      formattedValue: values?.formattedValue,
      rotation: values?.rotation
    };
  };

  const getBarDimensions = (context: Context) => {
    const barElement = context.chart.getDatasetMeta(context.datasetIndex).data[
      context.dataIndex
    ] as BarElement;

    const { width: barWidth, height: barHeight } = barElement.getProps(['width', 'height'], true);
    return { barWidth, barHeight };
  };

  const textWidthBuffer = 4;

  return {
    type: 'bar',
    label: yAxisItem.name,
    yAxisID: yAxisID || 'y',
    order,
    data: selectedDataset.source.map((item) => item[yAxisItem.index] as number),
    backgroundColor: colors[index % colors.length],
    borderRadius: (columnSetting?.barRoundness || 0) / 2,
    xAxisKeys,
    datalabels: {
      clamp: false,
      clip: false,
      labels: {
        barTotal: {
          display: (context) => {
            const rawValue = context.dataset.data[context.dataIndex] as number;
            if (!showLabels || !rawValue) return false;
            const { barWidth, barHeight } = getBarDimensions(context);
            if (barWidth < 13) return false;
            const formattedValue = formatBarAndLineDataLabel(
              rawValue,
              context,
              usePercentage,
              columnLabelFormat
            );
            const { width: widthOfFormattedValue } = context.chart.ctx.measureText(formattedValue);
            const rotation = widthOfFormattedValue > barWidth - textWidthBuffer ? -90 : 0;

            if (rotation === -90 && widthOfFormattedValue > barHeight - textWidthBuffer) {
              return false;
            }
            setBarDataLabelsManager(context, formattedValue, rotation);

            return 'auto';
          },
          formatter: (_, context) => {
            const formattedValue = getBarDataLabelsManager(context).formattedValue;
            return formattedValue;
          },
          rotation: (context) => {
            const { rotation } = getBarDataLabelsManager(context);
            return rotation;
          },
          color: dataLabelFontColorContrast,
          borderWidth: 0,
          padding: 1,
          borderRadius: 2.5,
          anchor: 'end',
          align: 'start',
          backgroundColor: ({ datasetIndex, chart }) => {
            const backgroundColor = chart.options.backgroundColor as string[];
            return backgroundColor[datasetIndex];
          }
        },
        ...dataLabelOptions
      }
    }
  } as ChartProps<'bar'>['data']['datasets'][number];
};

export const barSeriesBuilder_labels = (props: LabelBuilderProps) => {
  const { dataset, columnLabelFormats } = props;

  return dataset.source.map<string>((item) => {
    return formatChartLabelDelimiter(item[0] as string, columnLabelFormats);
  });
};
