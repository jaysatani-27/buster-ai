import { BusterChartProps } from '../../../interfaces';
import { formatChartLabelDelimiter, formatChartValueDelimiter } from '../../../commonHelpers';
import React, { useMemo } from 'react';
import { TooltipFormatterParams } from './interfaces';
import { busterAppStyleConfig } from '@/styles';
import { formatLabel } from '@/utils';
import { appendToKeyValueChain, extractFieldsFromChain } from '../../../chartHooks';
import last from 'lodash/last';
import type { ITooltipItem } from '../../../BusterChartTooltip/interfaces';
import { BusterChartTooltip } from '../../../BusterChartTooltip/BusterChartTooltip';

const token = busterAppStyleConfig.token!;

const FALLBACK_DOT_COLOR = token.colorSplit!;

export const BusterEChartTooltip: React.FC<{
  params: TooltipFormatterParams | TooltipFormatterParams[];
  columnLabelFormats: NonNullable<BusterChartProps['columnLabelFormats']>;
  tooltipKeys: string[];
  keyToUsePercentage: string[];
  useSeriesKeyValueAsLabel: boolean;
  useComboLegendDots: boolean;
}> = ({
  params,
  columnLabelFormats,
  tooltipKeys,
  keyToUsePercentage,
  useSeriesKeyValueAsLabel,
  useComboLegendDots
}) => {
  const arrayFormat = Array.isArray(params) ? params : [params];
  const firstParam = arrayFormat[0];
  const isScatter = firstParam.seriesType === 'scatter';
  const title = isScatter ? undefined : getTooltipTitle(firstParam, columnLabelFormats);

  const tooltipItems: ITooltipItem[] = useMemo(() => {
    if (isScatter) {
      const deliminatedKey = last(extractFieldsFromChain(firstParam.seriesName))?.key;
      const deliminatedKeyIsInTooltipKeys = tooltipKeys.includes(deliminatedKey!);
      //if we have multiple measures, we want to filter out the first key because it is the dataset name
      const filteredTooltipKeys = tooltipKeys.filter((key, index) => {
        if (deliminatedKey === key || index === 0) return true;
        return !deliminatedKeyIsInTooltipKeys;
      });

      const values = filteredTooltipKeys.map((dimensionName) => {
        const indexOfDimension = firstParam.dimensionNames.findIndex(
          (name) => name.toString() === dimensionName
        );
        const value = firstParam.value[indexOfDimension];

        return {
          formattedValue: formatChartValueDelimiter(value, dimensionName, columnLabelFormats),
          formattedLabel: formatChartLabelDelimiter(dimensionName, columnLabelFormats),
          formattedPercentage: undefined
        };
      });

      return [
        {
          usePercentage: false,
          color: firstParam.color,
          seriesType: firstParam.seriesType, //this is just for the dot,
          formattedLabel: formatChartLabelDelimiter(firstParam.seriesName, columnLabelFormats),
          values
        }
      ];
    }

    if (firstParam.seriesType === 'pie') {
      const firstParamSeriesName = firstParam.seriesName;
      return tooltipKeys
        .filter((dimensionName) => dimensionName === firstParamSeriesName)
        .map<ITooltipItem>((dimensionName, dimensionNameIndex) => {
          const indexOfDimension = dimensionNameIndex + 1; //encode.value[0] is the pie name
          const value = firstParam.value[indexOfDimension];
          const isActiveHover = dimensionName === firstParamSeriesName;
          const percentage = isActiveHover ? firstParam.percent : 0;

          let formattedPercentage = undefined;
          if (keyToUsePercentage.includes(dimensionName)) {
            formattedPercentage = percentageFormatter(
              percentage,
              dimensionName,
              columnLabelFormats
            );
          }

          return {
            usePercentage: keyToUsePercentage.includes(dimensionName),
            color: isActiveHover ? firstParam.color : FALLBACK_DOT_COLOR,
            seriesType: 'bar', //this is just for the dot,
            formattedLabel: formatChartLabelDelimiter(dimensionName, columnLabelFormats),
            values: [
              {
                formattedValue: formatChartValueDelimiter(value, dimensionName, columnLabelFormats),
                formattedLabel: formatChartLabelDelimiter(dimensionName, columnLabelFormats),
                formattedPercentage
              }
            ]
          };
        });
    }

    //BAR AND LINE
    return tooltipKeys.map<ITooltipItem>((dimensionName) => {
      const indexOfDimension = firstParam.dimensionNames.findIndex((name) => {
        return name.toString() === dimensionName;
      });

      const extractedDimensionName = extractFieldsFromChain(dimensionName);
      const firstDimensionName = appendToKeyValueChain(extractedDimensionName[0]);
      const deliminatedLabel = useSeriesKeyValueAsLabel ? dimensionName : firstDimensionName;

      const associatedSeries = arrayFormat.find((param) => {
        return param.seriesName === deliminatedLabel;
      });

      let formattedPercentage = undefined;
      if (keyToUsePercentage.includes(dimensionName)) {
        const percent = associatedSeries?.value[indexOfDimension] as number;
        formattedPercentage = percentageFormatter(percent, dimensionName, columnLabelFormats);
      }

      return {
        usePercentage: keyToUsePercentage.includes(dimensionName),
        color: associatedSeries?.color || FALLBACK_DOT_COLOR,
        seriesType: useComboLegendDots ? firstParam.seriesType : 'bar',
        formattedLabel: formatChartLabelDelimiter(deliminatedLabel, columnLabelFormats),
        values: [
          {
            formattedValue: formatChartValueDelimiter(
              firstParam.value[indexOfDimension],
              dimensionName,
              columnLabelFormats
            ),
            formattedLabel: formatChartLabelDelimiter(dimensionName, columnLabelFormats),
            formattedPercentage
          }
        ]
      };
    });
  }, []);

  return <BusterChartTooltip tooltipItems={tooltipItems} title={title} />;
};

const percentageFormatter = (
  percentage: number,
  axisKeyUnDeliminated: string,
  columnLabelFormats: NonNullable<BusterChartProps['columnLabelFormats']>
) => {
  const { key } = extractFieldsFromChain(axisKeyUnDeliminated)[0];
  let columnLabelFormat = columnLabelFormats[key];
  const isPercentage = columnLabelFormat?.style === 'percent';
  if (!isPercentage) {
    columnLabelFormat = {
      style: 'percent',
      columnType: 'number'
    };
  }
  return formatLabel(percentage, columnLabelFormat, false);
};

const getTooltipTitle = (
  firstParam: TooltipFormatterParams,
  columnLabelFormats: NonNullable<BusterChartProps['columnLabelFormats']>
) => {
  const axisTypeIsTime = firstParam.axisType === 'xAxis.time';
  //If we have a time axis, we want to format the date and add it as a title
  if (axisTypeIsTime) {
    const dimensionName = firstParam.dimensionNames[0].toString();
    const columnFormat = columnLabelFormats[dimensionName];
    const dimensionValue = firstParam.value[0];
    return formatLabel(dimensionValue, columnFormat, false);
  }

  return formatChartLabelDelimiter(firstParam.name, columnLabelFormats);
};
