import {
  Chart,
  ChartType,
  TooltipCallbacks,
  TooltipItem,
  ChartData,
  ParsingOptions
} from 'chart.js';

import { dataValue, setOriginalData, round, getPrecision, isObject } from './utils';
import { ExtendedChartData, ExtendedPlugin } from './types';

export const defaultStackKey = Symbol();

const getDataIndex = (
  labels: unknown[],
  data: unknown,
  parsing: ParsingOptions['parsing'],
  isHorizontal: boolean,
  srcIndex: number
) => {
  if (!isObject(data)) return srcIndex;

  const axis = isHorizontal ? 'y' : 'x';
  const parseKey = parsing && parsing[`${axis}AxisKey`];
  if (!parseKey) return srcIndex;
  const label = data[parseKey];
  if (!label) return srcIndex;
  const labelIndex = labels.findIndex((l) => l === label);

  return labelIndex < 0 ? srcIndex : labelIndex;
};

export const summarizeValues = (
  chartData: ChartData,
  visibles: number[],
  isHorizontal: boolean,
  individual: boolean,
  parsing?: ParsingOptions['parsing']
) => {
  const { labels = [], datasets } = chartData;
  const datasetDataLength = labels.length;

  const isStack = datasets?.[0]?.stack;
  const values = [...new Array(datasetDataLength)].map((el, i) => {
    return datasets.reduce(
      (sum, dataset, j) => {
        const parsingOptions = (dataset.parsing || parsing) as ParsingOptions['parsing'];
        const key = dataset.stack || defaultStackKey;
        const rec = dataset.data.find((ds, index) => {
          return getDataIndex(labels, ds, parsingOptions, isHorizontal, index) == i;
        });
        if (!sum[key]) sum[key] = 0;
        const value = Math.abs(dataValue(rec, isHorizontal, parsingOptions) || 0) * visibles[j];
        if (individual && !isStack) {
          if (sum[key] < value) sum[key] = value;
        } else {
          sum[key] += value;
        }

        return sum;
      },
      {} as { [key: string | symbol]: number }
    );
  });

  if (!isStack || !individual) return values;

  return values.map((rec) => {
    const maxVal = Math.max(...(Object.values(rec) as number[]));
    Object.keys(rec).forEach((key) => (rec[key] = maxVal));
    return rec;
  });
};

const isTargetDataset = (dataset: ChartData['datasets'][0], targetAxisId?: string) => {
  if (!targetAxisId) return true;

  // FIXME: avoid type error without any cast.
  const axisId = (dataset as any).xAxisID || (dataset as any).yAxisID;
  return axisId === targetAxisId;
};

const calculateRate = (
  data: ExtendedChartData,
  visibles: number[],
  isHorizontal: boolean,
  precision: number,
  individual: boolean,
  targetAxisId?: string,
  parsing?: ParsingOptions['parsing']
) => {
  const totals = summarizeValues(data, visibles, isHorizontal, individual, parsing);

  return data.datasets.map((dataset) => {
    const isTarget = isTargetDataset(dataset, targetAxisId);

    const ret = new Array(data.labels?.length || 0);
    dataset.data.forEach((val, j) => {
      const parsingOptions = (dataset.parsing || parsing) as ParsingOptions['parsing'];
      const dv = dataValue(val, isHorizontal, parsingOptions);
      const dataIndex = getDataIndex(data.labels || [], val, parsingOptions, isHorizontal, j);
      if (isTarget) {
        const key = dataset.stack || defaultStackKey;
        const total = totals[dataIndex]?.[key] || 1; //TODO there is a bug here where totals is undefined if there are multiple categories and the labels is short..
        if (!totals[dataIndex]?.[key]) {
          console.warn('bug!');
        }
        ret[dataIndex] = dv && total ? round(dv / total, precision) : 0;
      } else {
        ret[dataIndex] = dv;
      }
    });
    return ret;
  });
};

const tooltipLabel = (
  isHorizontal: boolean,
  targetAxisId?: string
): TooltipCallbacks<ChartType>['label'] => {
  return (tooltipItem: TooltipItem<ChartType>) => {
    const data = tooltipItem.chart.data;
    const datasetIndex = tooltipItem.datasetIndex;
    const index = tooltipItem.dataIndex;
    const datasetLabel = data.datasets[datasetIndex].label || '';
    const parsing = data.datasets[datasetIndex].parsing || tooltipItem.chart.options.parsing;
    const originalValue = data.originalData?.[datasetIndex]?.find(
      (rec, i) =>
        getDataIndex(
          data.labels || [],
          rec,
          parsing as ParsingOptions['parsing'],
          isHorizontal,
          i
        ) == index
    );
    const rateValue = data.calculatedData?.[datasetIndex]?.[index];
    const value = dataValue(originalValue, isHorizontal, parsing as ParsingOptions['parsing']);

    if (!isTargetDataset(data.datasets[datasetIndex], targetAxisId)) {
      return `${datasetLabel}: ${rateValue}`;
    }
    return `${datasetLabel}: ${rateValue}% (${value})`;
  };
};

const reflectData = (srcData: any[], datasets: ExtendedChartData['datasets']) => {
  if (!srcData) return;

  srcData.forEach((data, i) => {
    datasets[i].data = data;
  });
};

const isHorizontalChart = (chartInstance: Chart) => {
  return chartInstance.options.indexAxis === 'y';
};

const getTickOption = (hasNegative: boolean, fixNegativeScale: boolean) => {
  const baseOption = { max: 100 };
  if (!hasNegative) return { min: 0, ...baseOption };
  if (fixNegativeScale) return { min: -100, ...baseOption };
  return baseOption;
};

const setScaleOption = (
  chartInstance: Chart,
  axisId: string,
  stacked: boolean,
  tickOption: Record<string, any>
) => {
  const scaleOption = {
    stacked,
    ...tickOption,
    ...(chartInstance.options.scales?.[axisId] || {})
  };
  //@ts-ignore
  chartInstance.options.scales[axisId] = scaleOption;
};

export const beforeInit: ExtendedPlugin['beforeInit'] = (chartInstance, args, pluginOptions) => {
  if (!pluginOptions.enable) return;
  const { replaceTooltipLabel = true, fixNegativeScale = true, individual = false } = pluginOptions;

  const isHorizontal = isHorizontalChart(chartInstance);
  const targetAxis = isHorizontal ? 'x' : 'y';
  const hasNegative = chartInstance.data.datasets.some((dataset) => {
    return dataset.data.some(
      (value) =>
        (dataValue(
          value,
          isHorizontal,
          (dataset.parsing || chartInstance.options.parsing) as ParsingOptions['parsing']
        ) || 0) < 0
    );
  });
  const tickOption = getTickOption(hasNegative, fixNegativeScale);
  if (pluginOptions.axisId) {
    setScaleOption(chartInstance, pluginOptions.axisId, !individual, tickOption);
  } else {
    ['x', 'y'].forEach((axis) => {
      setScaleOption(chartInstance, axis, !individual, axis === targetAxis ? tickOption : {});
    });
  }

  // Replace tooltips
  if (!replaceTooltipLabel) return;

  //@ts-ignore
  chartInstance.options.plugins.tooltip.callbacks.label = tooltipLabel(
    isHorizontal,
    pluginOptions.axisId
  );
};

export const beforeUpdate: ExtendedPlugin['beforeUpdate'] = (
  chartInstance,
  _args,
  pluginOptions
) => {
  if (!pluginOptions.enable) return;

  const data = chartInstance.data;

  setOriginalData(data);
  const visibles = data.datasets.map((dataset, i) =>
    (chartInstance.getDatasetMeta(i)?.hidden ?? dataset.hidden) ? 0 : 1
  );
  const precision = getPrecision(pluginOptions);
  data.calculatedData = calculateRate(
    data,
    visibles,
    isHorizontalChart(chartInstance),
    precision,
    pluginOptions.individual || false,
    pluginOptions.axisId,
    chartInstance.options.parsing as ParsingOptions['parsing']
  );
  reflectData(data.calculatedData, data.datasets);
};

export const afterUpdate: ExtendedPlugin['afterUpdate'] = (chartInstance, _args, pluginOptions) => {
  if (!pluginOptions.enable) return;

  const data = chartInstance.data;

  reflectData(data.originalData || [], data.datasets);
};
