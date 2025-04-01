import type { ChartProps } from '../../core';
import type { ChartSpecificOptionsProps } from './interfaces';
import type { ChartType as ChartJSChartType } from 'chart.js';
import type { DeepPartial } from 'utility-types';
import type { PluginChartOptions } from 'chart.js';
import { AnnotationPluginOptions } from 'chartjs-plugin-annotation';
import { ArrayOperations } from '@/utils/math';
import { formatLabel } from '@/utils/columnFormatter';
import { busterAppStyleConfig } from '@/styles/busterAntDStyleConfig';
import { getPieInnerLabelTitle } from '../../../commonHelpers';
import { ChartJSOrUndefined } from '../../core/types';
import {
  BusterChartConfigProps,
  BusterChartProps,
  ColumnLabelFormat
} from '@/components/charts/interfaces';
import { determineFontColorContrast } from '@/utils/colors';
import { Context } from 'chartjs-plugin-datalabels';
const token = busterAppStyleConfig.token!;

type PieOptions = ChartProps<'pie'>['options'] | ChartProps<'doughnut'>['options'];

export const pieOptionsHandler = ({
  pieDonutWidth = 0
}: ChartSpecificOptionsProps): ChartProps<ChartJSChartType>['options'] => {
  const result: PieOptions = {
    cutout: pieDonutWidth ? `${pieDonutWidth + 35}%` : 0
  };

  return result as ChartProps<ChartJSChartType>['options'];
};

export const piePluginsHandler = ({
  pieInnerLabelTitle,
  pieInnerLabelAggregate = 'sum',
  pieShowInnerLabel,
  columnLabelFormats,
  pieLabelPosition,
  pieDisplayLabelAs,
  selectedAxis,
  pieDonutWidth
}: ChartSpecificOptionsProps): DeepPartial<PluginChartOptions<ChartJSChartType>>['plugins'] => {
  let returnValue: DeepPartial<PluginChartOptions<ChartJSChartType>>['plugins'] = {};

  if (pieShowInnerLabel && pieDonutWidth !== 0) {
    const annotation: AnnotationPluginOptions = {
      annotations: {
        donutInnerLabel: {
          display: pieShowInnerLabel,
          type: 'doughnutLabel' as 'label', //this is incorrectly typed in the package...
          content: ({ chart }) => {
            const firstDatasetIndex = 0; //we can assume there is only one dataset
            const datasets = chart.data.datasets;
            const firstDatasetData = datasets[firstDatasetIndex].data as number[];
            return [
              getPieInnerLabelTitle(pieInnerLabelTitle, pieInnerLabelAggregate),
              getInnerLabelValue(
                chart,
                firstDatasetData,
                pieInnerLabelAggregate,
                selectedAxis,
                columnLabelFormats
              )
            ];
          },
          font: [{ size: 14 }, { size: 22 }],
          color: [token.colorTextTertiary!, token.colorTextBase!]
        }
      }
    };
    returnValue = { annotation };
  }

  const usePercent = pieDisplayLabelAs === 'percent';
  const numberOfYAxis = selectedAxis.y.length;

  returnValue = {
    ...returnValue,
    outlabels:
      pieLabelPosition === 'outside' && numberOfYAxis === 1
        ? {
            display: true,
            usePercent,
            formatter: (value: number) => {
              return labelFormatter(value, selectedAxis, columnLabelFormats, usePercent);
            }
          }
        : false,
    datalabels: {
      display: pieLabelPosition === 'inside',
      anchor: 'center',
      backgroundColor: ({ dataIndex, chart }) => {
        const backgroundColor = chart.options.backgroundColor as string[];
        return backgroundColor[dataIndex];
      },
      borderWidth: 0,
      borderRadius: 4,
      color: ({ dataIndex, chart }) => {
        const backgroundColor = chart.options.backgroundColor as string[];
        const color = backgroundColor[dataIndex];
        return determineFontColorContrast(color);
      },
      padding: 2,
      formatter: (value: number, context) => {
        return labelFormatter(
          usePercent ? percentFormatter(context, value) : value,
          selectedAxis,
          columnLabelFormats,
          usePercent
        );
      }
    }
  };

  return returnValue;
};

const getInnerLabelValue = (
  chart: ChartJSOrUndefined,
  firstDatasetData: number[],
  pieInnerLabelAggregate: BusterChartConfigProps['pieInnerLabelAggregate'] = 'sum',
  selectedAxis: ChartSpecificOptionsProps['selectedAxis'],
  columnLabelFormats: NonNullable<BusterChartProps['columnLabelFormats']>
): string => {
  try {
    const activeLegendItems =
      chart?.legend?.legendItems?.filter((item) => item.hidden === false) || [];
    const dataByActiveLegendItems = activeLegendItems.map((item) => {
      const index = item.index as number;
      return firstDatasetData[index];
    });
    const operator = new ArrayOperations(dataByActiveLegendItems);
    const result = operator[pieInnerLabelAggregate]();

    if (pieInnerLabelAggregate === 'count') {
      return result.toString();
    }

    const yColumn = selectedAxis.y[0]!;
    const yColumnLabel = columnLabelFormats[yColumn];
    const formattedLabel = formatLabel(result, yColumnLabel);
    return formattedLabel;
  } catch (error) {
    return '';
  }
};

const percentStyle: ColumnLabelFormat = {
  columnType: 'number',
  style: 'percent'
};

const labelFormatter = (
  value: number,
  selectedAxis: ChartSpecificOptionsProps['selectedAxis'],
  columnLabelFormats: NonNullable<BusterChartProps['columnLabelFormats']>,
  usePercent: boolean
) => {
  if (usePercent) {
    return formatLabel(value, percentStyle);
  }

  const yColumn = selectedAxis.y[0]!;
  const yColumnLabel = columnLabelFormats[yColumn];
  const formattedLabel = formatLabel(value, yColumnLabel);
  return formattedLabel;
};

const percentFormatter = (context: Context, value: number) => {
  const total = (context.dataset.data as number[]).reduce(
    (sum: number, val: number) => sum + val,
    0
  );
  value = (value / total) * 100;

  return value;
};
