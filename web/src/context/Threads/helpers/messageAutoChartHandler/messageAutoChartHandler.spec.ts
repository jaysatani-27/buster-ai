import { type BusterThreadMessage, DEFAULT_CHART_CONFIG } from '@/api/buster_rest/threads';
import { createDefaultChartConfig } from '.';

describe('createDefaultChartConfig', () => {
  it('should create a default chart config', () => {
    const message = createTestMessage(DEFAULT_CHART_CONFIG);

    const config = createDefaultChartConfig(message);

    const expected = {
      colors: [
        '#B399FD',
        '#FC8497',
        '#FBBC30',
        '#279EFF',
        '#E83562',
        '#41F8FF',
        '#F3864F',
        '#C82184',
        '#31FCB4',
        '#E83562'
      ],
      selectedChartType: 'table',
      selectedView: 'table',
      yAxisShowAxisLabel: true,
      yAxisShowAxisTitle: true,
      yAxisAxisTitle: null,
      yAxisStartAxisAtZero: null,
      yAxisScaleType: 'linear',
      y2AxisShowAxisLabel: true,
      y2AxisAxisTitle: null,
      y2AxisShowAxisTitle: true,
      y2AxisStartAxisAtZero: true,
      y2AxisScaleType: 'linear',
      xAxisShowAxisLabel: true,
      xAxisShowAxisTitle: true,
      xAxisAxisTitle: null,
      xAxisLabelRotation: 'auto',
      xAxisDataZoom: false,
      categoryAxisTitle: null,
      showLegend: null,
      gridLines: true,
      goalLines: [],
      trendlines: [],
      showLegendHeadline: false,
      disableTooltip: false,
      barAndLineAxis: { x: [], y: [], tooltip: null, category: [] },
      scatterAxis: { x: [], y: [], size: [], tooltip: null, category: undefined },
      comboChartAxis: { x: [], y: [], y2: [], tooltip: null, category: undefined },
      pieChartAxis: { x: [], y: [], tooltip: null },
      lineGroupType: null,
      scatterDotSize: [3, 15],
      barSortBy: [],
      barLayout: 'vertical',
      barGroupType: 'group',
      barShowTotalAtTop: false,
      pieShowInnerLabel: true,
      pieInnerLabelAggregate: 'sum',
      pieInnerLabelTitle: 'Total',
      pieLabelPosition: null,
      pieDonutWidth: 40,
      pieMinimumSlicePercentage: 0,
      pieDisplayLabelAs: 'number',
      metricColumnId: 'test',
      metricValueAggregate: 'sum',
      metricHeader: null,
      metricSubHeader: null,
      metricValueLabel: null,
      tableColumnOrder: null,
      tableColumnWidths: null,
      tableHeaderBackgroundColor: null,
      tableHeaderFontColor: null,
      tableColumnFontColor: null,
      columnSettings: {
        test: {
          showDataLabels: false,
          columnVisualization: 'bar',
          lineWidth: 2,
          lineStyle: 'line',
          lineType: 'normal',
          lineSymbolSize: 0,
          barRoundness: 8,
          showDataLabelsAsPercentage: false
        }
      },
      columnLabelFormats: {
        test: {
          style: 'string',
          compactNumbers: false,
          columnType: 'string',
          displayName: '',
          numberSeparatorStyle: ',',
          minimumFractionDigits: 0,
          maximumFractionDigits: 2,
          currency: 'USD',
          convertNumberTo: null,
          dateFormat: 'auto',
          useRelativeTime: false,
          isUTC: false,
          multiplier: 1,
          prefix: '',
          suffix: '',
          replaceMissingDataWith: null,
          makeLabelHumanReadable: true
        }
      }
    };

    expect(config).toEqual(expected);
  });
});

const TEST_DATA_METADATA: BusterThreadMessage['data_metadata'] = {
  column_count: 1,
  column_metadata: [
    {
      name: 'test',
      simple_type: 'text',
      min_value: 0,
      max_value: 100,
      unique_values: 10,
      type: 'text'
    }
  ],
  row_count: 10
};

const TEST_MESSAGE = {
  chart_config: DEFAULT_CHART_CONFIG,
  data_metadata: TEST_DATA_METADATA
};

const createTestMessage = (chartConfig: BusterThreadMessage['chart_config']) => {
  return { ...TEST_MESSAGE };
};
