import { DATASET_IDS, TrendlineDataset } from '@/components/charts/chartHooks';
import type { BusterChartConfigProps, ChartType, Trendline } from '@/components/charts/interfaces';
import { AnnotationOptions, AnnotationPluginOptions } from 'chartjs-plugin-annotation';
import { useMemo } from 'react';
import { defaultLabelOptionConfig } from '../useChartSpecificOptions/labelOptionConfig';
import { formatLabel } from '@/utils';
import { TypeToLabel } from '@/app/app/_controllers/ThreadController/ThreadControllerEditContent/SidebarStylingApp/StylingAppStyling/EditTrendline/config';
import { ChartProps } from '../../core';

export const useTrendlines = ({
  trendlines,
  columnLabelFormats,
  selectedChartType
}: {
  trendlines: TrendlineDataset[];
  selectedChartType: ChartType;
  columnLabelFormats: NonNullable<BusterChartConfigProps['columnLabelFormats']>;
}): {
  trendlineAnnotations: AnnotationPluginOptions['annotations'];
  trendlineSeries: ChartProps<'line'>['data']['datasets'][number][];
} => {
  const canSupportTrendlines = useMemo(() => {
    return selectedChartType === 'line' || selectedChartType === 'scatter';
  }, [selectedChartType]);

  const annotationTrendlines = useMemo(() => {
    if (!canSupportTrendlines) return [];
    return trendlines.filter(
      (trendline) => annotationTypes.includes(trendline.type) && trendline.show
    );
  }, [trendlines, canSupportTrendlines]);

  const seriesTrendlines = useMemo(() => {
    return trendlines.filter(
      (trendline) => !annotationTypes.includes(trendline.type) && trendline.show
    );
  }, [trendlines, canSupportTrendlines]);

  const trendlineAnnotations: AnnotationPluginOptions['annotations'] = useMemo(() => {
    return annotationTrendlines.reduce<Record<string, AnnotationOptions<'line'>>>(
      (acc, trendline) => {
        const name = trendline.type;
        const builderResult = annotationBuilder[name](trendline);
        const value = trendline.source[0][0] as number;
        const formattedValue = formatLabel(value, columnLabelFormats[trendline.columnId]);
        const trendlineLabel = trendline.trendlineLabel || TypeToLabel[trendline.type];
        const labelContent = trendlineLabel
          ? `${trendlineLabel}: ${formattedValue}`
          : formattedValue;
        return {
          ...acc,
          [name]: {
            ...builderResult,
            type: 'line',
            borderColor: trendline.trendLineColor || 'black',
            borderWidth: 1.5,
            label: {
              content: labelContent,
              display: trendline.showTrendlineLabel,
              ...defaultLabelOptionConfig
            },
            scaleID: 'y'
          }
        };
      },
      {}
    );
  }, [annotationTrendlines, canSupportTrendlines]);

  const trendlineSeries: ChartProps<'line'>['data']['datasets'][number][] = useMemo(() => {
    const series = seriesTrendlines.map<ChartProps<'line'>['data']['datasets'][number]>(
      ({
        id,
        source,
        trendLineColor,
        trendlineLabel: trendlineLabelProp,
        showTrendlineLabel,
        equation
      }) => {
        return {
          type: 'line',
          data: source.map((i) => i[1] as number),
          borderColor: trendLineColor || 'black',
          borderWidth: 2,
          isTrendline: true,
          pointHoverRadius: 0,
          pointRadius: 0,
          yAxisID: 'y',
          stack: id,
          tension: 0.35,
          order: -1,
          datalabels: showTrendlineLabel
            ? {
                ...defaultLabelOptionConfig,
                anchor: 'end',
                align: 'left',
                display: (context) => {
                  const datasetLength = context.dataset.data.length;
                  return context.dataIndex === datasetLength - 1;
                },
                formatter: () => {
                  const trendlineLabel = trendlineLabelProp ? trendlineLabelProp : equation;
                  return `${trendlineLabel}`;
                },
                yAdjust: -10
              }
            : undefined
        };
      }
    );

    return series;
  }, [seriesTrendlines, canSupportTrendlines]);

  return {
    trendlineAnnotations,
    trendlineSeries
  };
};

type TrendlineType = Trendline['type'];

const annotationTypes: TrendlineType[] = ['average', 'min', 'max', 'median'];

const annotationBuilder: Record<
  TrendlineType,
  (trendline: TrendlineDataset) => AnnotationOptions<'line'> | null
> = {
  average: (trendline) => ({
    type: 'line',
    value: trendline.source[0][0] as number
  }),
  min: (trendline) => ({
    type: 'line',
    value: trendline.source[0][0] as number
  }),
  max: (trendline) => ({
    type: 'line',
    value: trendline.source[0][0] as number
  }),
  median: (trendline) => ({
    type: 'line',
    value: trendline.source[0][0] as number
  }),
  linear_regression: (trendline) => {
    const isLinearSlope = trendline.trendlineLabel === DATASET_IDS.linearSlope(trendline.columnId);

    if (!isLinearSlope) {
      return null;
    }

    return null;
  },
  logarithmic_regression: (trendline) => {
    return null;
  },
  exponential_regression: (trendline) => {
    return null;
  },
  polynomial_regression: (trendline) => {
    return null;
  }
};
