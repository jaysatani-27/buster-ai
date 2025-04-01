import {
  BusterChartConfigProps,
  ChartType,
  ColumnLabelFormat,
  GoalLine
} from '@/components/charts/interfaces';
import { AnnotationOptions, AnnotationPluginOptions } from 'chartjs-plugin-annotation';
import { useMemo } from 'react';
import { ChartOptions, PluginChartOptions } from 'chart.js';
import { busterAppStyleConfig } from '@/styles/busterAntDStyleConfig';
import { formatLabel } from '@/utils/columnFormatter';
import { yAxisSimilar } from '@/components/charts/commonHelpers';
import { extractFieldsFromChain } from '@/components/charts/chartHooks';
import { defaultLabelOptionConfig } from '../useChartSpecificOptions/labelOptionConfig';

const token = busterAppStyleConfig.token!;

export const useGoalLines = ({
  goalLines,
  selectedChartType,
  columnLabelFormats,
  yAxisKeys,
  y2AxisKeys
}: {
  goalLines: GoalLine[];
  selectedChartType: ChartType;
  columnLabelFormats: NonNullable<BusterChartConfigProps['columnLabelFormats']>;
  yAxisKeys: string[];
  y2AxisKeys: string[] | undefined;
}): AnnotationPluginOptions['annotations'] => {
  const canSupportGoalLines = useMemo(() => {
    return selectedChartType === 'line' || selectedChartType === 'scatter';
  }, [selectedChartType]);

  const trendlineLabelFormat: ColumnLabelFormat = useMemo(() => {
    const allKeys = [...yAxisKeys, ...(y2AxisKeys || [])];
    const isSimilar = yAxisSimilar(allKeys, columnLabelFormats);
    if (isSimilar) {
      const key = extractFieldsFromChain(yAxisKeys[0]!)[0]?.key;
      return columnLabelFormats[key!];
    }
    return {
      columnType: 'number',
      style: 'number'
    };
  }, [columnLabelFormats, yAxisKeys, y2AxisKeys]);

  const annotations: AnnotationPluginOptions['annotations'] = useMemo(() => {
    return goalLines.reduce<Record<string, AnnotationOptions<'line'>>>((acc, goalLine, index) => {
      const { value, goalLineLabel, goalLineColor, showGoalLineLabel, show } = goalLine;
      if (show) {
        const id = `🔥-goal-line-${index}`;
        const formattedValue = [goalLineLabel, formatLabel(value, trendlineLabelFormat)]
          .filter(Boolean)
          .join(' ');

        const item: AnnotationOptions<'line'> = {
          type: 'line',
          id,
          yMin: value,
          yMax: value,
          borderColor: goalLineColor || 'black',
          borderWidth: 1.5,
          borderDash: [5, 5],
          label: {
            content: formattedValue,
            display: showGoalLineLabel,
            //@ts-ignore
            anchor: 'end',
            align: 'top',
            ...defaultLabelOptionConfig
          }
        };
        acc[id] = item;
      }
      return acc;
    }, {});
  }, [goalLines, canSupportGoalLines, trendlineLabelFormat]);

  return annotations;
};
