import React, { useMemo } from 'react';
import { BusterChartLegendWrapper } from '../BusterChartLegend/BusterChartLegendWrapper';
import { EChartsInstance } from 'echarts-for-react';
import { useBusterEChartLegend } from './hooks';
import {
  DEFAULT_CATEGORY_AXIS_COLUMN_NAMES,
  DEFAULT_X_AXIS_COLUMN_NAMES,
  DEFAULT_Y_AXIS_COLUMN_NAMES
} from '../BusterChartLegend';
import { BusterChartProps, ChartEncodes, ComboChartAxis, ScatterAxis } from '../interfaces';

interface BusterEChartLegendWrapperProps {
  children: React.ReactNode;
  animate: boolean;
  loading: boolean;
  columnLabelFormats: NonNullable<BusterChartProps['columnLabelFormats']>;
  selectedAxis: ChartEncodes | undefined;
  chartMounted: boolean;
  showLegend: BusterChartProps['showLegend'] | undefined;
  showLegendHeadline: BusterChartProps['showLegendHeadline'];
  className: string | undefined;
  selectedChartType: NonNullable<BusterChartProps['selectedChartType']>;
  columnSettings: NonNullable<BusterChartProps['columnSettings']>;
  columnMetadata: NonNullable<BusterChartProps['columnMetadata']>;
  lineGroupType: BusterChartProps['lineGroupType'];
  barGroupType: BusterChartProps['barGroupType'];
  colors: NonNullable<BusterChartProps['colors']>;
  chartRef: React.RefObject<EChartsInstance>;
}

export const BusterEChartLegendWrapper: React.FC<BusterEChartLegendWrapperProps> = ({
  children,
  className = '',
  loading,
  showLegend: showLegendProp,
  chartMounted,
  columnLabelFormats,
  selectedAxis,
  chartRef,
  selectedChartType,
  animate,
  columnSettings,
  columnMetadata,
  showLegendHeadline,
  lineGroupType,
  barGroupType,
  colors
}) => {
  const {
    renderLegend,
    legendItems,
    showLegend,
    onHoverItem,
    onLegendItemClick,
    onLegendItemFocus,
    inactiveDatasets
  } = useBusterEChartLegend({
    selectedAxis,
    columnLabelFormats,
    chartMounted,
    chartRef,
    selectedChartType,
    showLegend: showLegendProp,
    showLegendHeadline,
    columnSettings,
    columnMetadata,
    lineGroupType,
    barGroupType,
    colors,
    loading
  });

  return (
    <BusterChartLegendWrapper
      className={className}
      animate={animate}
      renderLegend={renderLegend}
      legendItems={legendItems}
      showLegend={showLegend}
      showLegendHeadline={showLegendHeadline}
      inactiveDatasets={inactiveDatasets}
      onHoverItem={onHoverItem}
      onLegendItemClick={onLegendItemClick}
      onLegendItemFocus={onLegendItemFocus}>
      {children}
    </BusterChartLegendWrapper>
  );
};
