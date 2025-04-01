import type { ChartProps } from '../../core';
import type { ChartType as ChartJSChartType } from 'chart.js';
import type { DeepPartial } from 'utility-types';
import type { PluginChartOptions } from 'chart.js';
import { ChartSpecificOptionsProps } from './interfaces';
import { busterAppStyleConfig } from '@/styles/busterAntDStyleConfig';
import { AnnotationPluginOptions } from 'chartjs-plugin-annotation';
import { ChartEncodes } from '@/components/charts/interfaces';
import { defaultLabelOptionConfig } from './labelOptionConfig';
import { formatYAxisLabel } from '@/components/charts/commonHelpers';

const token = busterAppStyleConfig.token!;

export const barOptionsHandler = (
  props: ChartSpecificOptionsProps
): ChartProps<ChartJSChartType>['options'] => {
  return {};
};

export const barPluginsHandler = ({
  barShowTotalAtTop,
  barGroupType,
  columnLabelFormats,
  selectedAxis,
  data,
  ...rest
}: ChartSpecificOptionsProps): DeepPartial<PluginChartOptions<ChartJSChartType>>['plugins'] => {
  return {
    totalizer: {
      enabled: barShowTotalAtTop && barGroupType === 'stack'
    },
    annotation: {
      annotations: barShowTotalAtTop
        ? getTotalBarAnnotations(data, selectedAxis, columnLabelFormats)
        : undefined
    }
  };
};

const getTotalBarAnnotations = (
  data: ChartProps<ChartJSChartType>['data'],
  selectedAxis: ChartEncodes,
  columnLabelFormats: ChartSpecificOptionsProps['columnLabelFormats']
): AnnotationPluginOptions['annotations'] => {
  const datasets = data.datasets;
  const labels = data.labels!;
  const shownDatasets = datasets.filter((dataset) => !dataset.hidden && !dataset.isTrendline);
  const lastShownDataset = shownDatasets[shownDatasets.length - 1];
  const yAxis = selectedAxis.y;

  const annotations: AnnotationPluginOptions['annotations'] = {};

  // labels.forEach((label, labelIndex) => {
  //   annotations[labelIndex] = {
  //     ...defaultLabelOptionConfig,
  //     position: {
  //       x: 'center',
  //       y: 'end'
  //     },
  //     yAdjust: -3,

  //     type: 'label',
  //     content: (context) => {
  //       const rawTotal = context.chart.$totalizer.stackTotals[labelIndex];
  //       const formattedTotal = formatYAxisLabel(
  //         rawTotal,
  //         yAxis,
  //         true,
  //         columnLabelFormats,
  //         false,
  //         false
  //       );
  //       return formattedTotal;
  //     },
  //     xValue: labelIndex, // This corresponds to the first stack
  //     yValue: (context) => {
  //       const total = context.chart.$totalizer.stackTotals[labelIndex];
  //       return total;
  //     }
  //   };
  // });

  return annotations;
};
