'use client';

import { useMemo } from 'react';
import {
  BusterChartConfigProps,
  BusterChartProps,
  ChartType,
  BarSortBy,
  ChartEncodes,
  ScatterAxis,
  Trendline,
  ComboChartAxis,
  IColumnLabelFormat
} from '../../interfaces';
import uniq from 'lodash/uniq';
import {
  getLineBarPieDatasetOptions,
  getLineBarPieDimensions,
  getLineBarPieTooltipKeys,
  getLineBarPieYAxisKeys,
  mapLineBarPieData,
  processLineBarData,
  sortLineBarData
} from './datasetHelpers_BarLinePie';
import {
  getScatterDatasetOptions,
  getScatterDimensions,
  getScatterTooltipKeys,
  mapScatterData,
  processScatterData
} from './datasetHelpers_Scatter';
import { TrendlineDataset, useDataTrendlineOptions } from './useDataTrendlineOptions';
import { DatasetOption } from './interfaces';

type DatasetHookResult = {
  datasetOptions: DatasetOption[];
  dataTrendlineOptions: TrendlineDataset[];
  yAxisKeys: string[];
  y2AxisKeys: string[];
  tooltipKeys: string[];
  hasMismatchedTooltipsAndMeasures: boolean;
};

type DatasetHookParams = {
  data: NonNullable<BusterChartProps['data']>;
  barSortBy?: BarSortBy;
  groupByMethod?: BusterChartProps['groupByMethod'];
  selectedAxis: ChartEncodes;
  selectedChartType: ChartType;
  pieMinimumSlicePercentage: NonNullable<BusterChartProps['pieMinimumSlicePercentage']> | undefined;
  columnLabelFormats: NonNullable<BusterChartConfigProps['columnLabelFormats']>;
  barGroupType: BusterChartProps['barGroupType'] | undefined;
  lineGroupType: BusterChartProps['lineGroupType'];
  trendlines: Trendline[] | undefined;
};

const defaultYAxis2 = [] as string[];

export const useDatasetOptions = (params: DatasetHookParams): DatasetHookResult => {
  const {
    selectedAxis,
    data,
    selectedChartType,
    barSortBy,
    columnLabelFormats,
    pieMinimumSlicePercentage,
    barGroupType,
    lineGroupType,
    trendlines
  } = params;
  const {
    x: xFields,
    y: yAxisFields,
    size: sizeField,
    tooltip: _tooltipFields = null,
    category: categoryFields = []
  } = selectedAxis as ScatterAxis;
  const { y2: y2AxisFields = defaultYAxis2 } = selectedAxis as ComboChartAxis;

  const tooltipFields = useMemo(() => _tooltipFields || [], [_tooltipFields]);

  const isPieChart = selectedChartType === 'pie';
  const isBarChart = selectedChartType === 'bar';
  const isScatter = selectedChartType === 'scatter';
  const isComboChart = selectedChartType === 'combo';
  const xAxisField = xFields[0];

  const xFieldsString = useMemo(() => xFields.join(','), [xFields]);
  const yAxisFieldsString = useMemo(() => yAxisFields.join(','), [yAxisFields]);
  const y2AxisFieldsString = useMemo(() => y2AxisFields.join(','), [y2AxisFields]);
  const categoryFieldsString = useMemo(() => categoryFields.join(','), [categoryFields]);
  const sizeFieldString = useMemo(() => sizeField?.join(','), [sizeField]);
  const tooltipFieldsString = useMemo(() => tooltipFields.join(','), [tooltipFields]);

  const xFieldColumnLabelFormatColumnTypes: IColumnLabelFormat['columnType'][] = useMemo(() => {
    return xFields.map((field) => columnLabelFormats[field]?.columnType);
  }, [xFieldsString, columnLabelFormats]);

  //WILL ONLY BE USED FOR BAR CHART
  const xFieldSorts = useMemo(() => {
    if (isScatter) return [];
    if (isPieChart) return [];
    if (isBarChart && barSortBy && barSortBy?.some((y) => y !== 'none')) return [];
    return xFieldColumnLabelFormatColumnTypes.filter((columnType) => columnType === 'date');
  }, [xFieldColumnLabelFormatColumnTypes, isPieChart, isBarChart, isScatter, barSortBy]);

  const xFieldSortsString = useMemo(() => xFieldSorts.join(','), [xFieldSorts]);

  const measureFields: string[] = useMemo(() => {
    return uniq([...yAxisFields, ...y2AxisFields, ...tooltipFields]);
  }, [yAxisFieldsString, y2AxisFieldsString, tooltipFieldsString]);

  const sortedData = useMemo(() => {
    if (isScatter) return data;
    return sortLineBarData(data, xFieldSorts, xFields);
  }, [data, xFieldSortsString, xFieldsString, isScatter]);

  const { dataMap, xValuesSet, categoriesSet } = useMemo(() => {
    if (isScatter) return mapScatterData(sortedData, categoryFields);
    return mapLineBarPieData(sortedData, xFields, categoryFields, measureFields);
  }, [sortedData, xFieldsString, categoryFieldsString, measureFields, isScatter]);

  const measureFieldsReplaceDataWithKey = useMemo(() => {
    return measureFields
      .map((field) => {
        const value = columnLabelFormats[field]?.replaceMissingDataWith;
        if (value === undefined) return 0;
        if (value === null) return null;
        if (value === '') return '';
        return value;
      })
      .join(',');
  }, [measureFields.join(''), columnLabelFormats]);

  const processedData = useMemo(() => {
    if (isScatter) {
      return processScatterData(
        data,
        xAxisField,
        measureFields,
        categoryFields,
        sizeField,
        columnLabelFormats,
        categoriesSet
      );
    }

    return processLineBarData(
      categoriesSet,
      xValuesSet,
      dataMap,
      measureFields,
      columnLabelFormats
    );
  }, [
    data,
    xFieldSortsString,
    xFieldsString,
    isScatter,
    categoriesSet,
    xValuesSet,
    dataMap,
    measureFields,
    sizeFieldString,
    measureFieldsReplaceDataWithKey //use this instead of columnLabelFormats
  ]);

  const dimensions: string[] = useMemo(() => {
    if (isScatter) {
      return getScatterDimensions(categoriesSet, xAxisField, measureFields, sizeField);
    }
    return getLineBarPieDimensions(categoriesSet, measureFields, xFields);
  }, [categoriesSet, measureFields, xFieldsString, sizeFieldString, isScatter]);

  const yAxisKeys = useMemo(() => {
    if (isScatter) return getLineBarPieYAxisKeys(categoriesSet, yAxisFields); //not a typo. I want to use the same function for both scatter and bar/line/pie
    return getLineBarPieYAxisKeys(categoriesSet, yAxisFields);
  }, [categoriesSet, yAxisFieldsString, isScatter]);

  const y2AxisKeys = useMemo(() => {
    if (!isComboChart) return defaultYAxis2;
    return getLineBarPieYAxisKeys(categoriesSet, y2AxisFields);
  }, [categoriesSet, y2AxisFieldsString, isComboChart]);

  const tooltipKeys = useMemo(() => {
    if (isScatter) {
      return getScatterTooltipKeys(
        tooltipFields,
        xAxisField,
        categoriesSet,
        measureFields,
        sizeField
      );
    }
    return getLineBarPieTooltipKeys(categoriesSet, tooltipFields, measureFields);
  }, [categoriesSet, tooltipFieldsString, xAxisField, sizeFieldString, isScatter]);

  const hasMismatchedTooltipsAndMeasures = useMemo(() => {
    const allYAxis = [...yAxisFields, ...y2AxisFields];
    if (tooltipFields.length === 0) return false;
    return tooltipFields.some((yAxis) => {
      return !allYAxis.includes(yAxis);
    });
  }, [yAxisFields, y2AxisFields, tooltipFieldsString]);

  const datasetOptions = useMemo(() => {
    if (isScatter) {
      return getScatterDatasetOptions(processedData, dimensions);
    }

    return getLineBarPieDatasetOptions(
      dimensions,
      processedData as (string | number | Date | null)[][],
      selectedChartType,
      pieMinimumSlicePercentage,
      barSortBy,
      yAxisKeys,
      xFieldSorts,
      barGroupType,
      lineGroupType
    );
  }, [
    lineGroupType,
    barGroupType,
    processedData,
    dimensions,
    yAxisKeys,
    barSortBy,
    pieMinimumSlicePercentage,
    measureFields,
    isScatter,
    selectedChartType
  ]);

  const dataTrendlineOptions = useDataTrendlineOptions({
    datasetOptions,
    trendlines,
    selectedAxis,
    selectedChartType,
    columnLabelFormats
  });

  return {
    datasetOptions,
    dataTrendlineOptions,
    yAxisKeys,
    y2AxisKeys,
    tooltipKeys,
    hasMismatchedTooltipsAndMeasures
  };
};
