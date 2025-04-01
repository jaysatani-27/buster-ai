import React, { useMemo } from 'react';
import { useMount } from 'ahooks';
import { formatLabel, JsonDataFrameOperationsSingle, timeout } from '@/utils';
import { motion, AnimatePresence } from 'framer-motion';
import { Title } from '@/components';
import { ColumnLabelFormat, MetricChartProps } from '../interfaces';
import { DEFAULT_COLUMN_LABEL_FORMAT } from '@/api/buster_rest';
import type { BusterMetricChartProps } from './interfaces';

export const BusterMetricChart: React.FC<BusterMetricChartProps> = React.memo(
  ({
    className = '',
    onMounted,
    metricColumnId,
    metricHeader,
    metricSubHeader,
    metricValueAggregate,
    data,
    isDarkMode,
    animate,
    columnLabelFormats,
    metricValueLabel,
    onInitialAnimationEnd
  }) => {
    const firstRow = data?.[0];
    const firstRowValue = firstRow?.[metricColumnId];
    const yLabelFormat = columnLabelFormats[metricColumnId];

    const canUseAggregateForHeaders = useMemo(() => {
      return data.length === 1 && metricValueAggregate === 'first';
    }, [data.length, metricValueAggregate]);

    const headerColumnLabelFormat: ColumnLabelFormat = useMemo(() => {
      const isDerivedTitle = typeof metricHeader === 'object' && metricHeader?.columnId;
      if (isDerivedTitle) {
        return columnLabelFormats[metricHeader.columnId];
      }
      return DEFAULT_COLUMN_LABEL_FORMAT;
    }, [metricHeader, columnLabelFormats]);

    const headerLabelFormat: ColumnLabelFormat = useMemo(() => {
      const isDerivedTitle = typeof metricHeader === 'object' && metricHeader?.columnId;
      if (isDerivedTitle) {
        const isCount = metricValueAggregate === 'count';
        const columnLabelFormat = headerColumnLabelFormat;
        const format: ColumnLabelFormat = {
          ...columnLabelFormat,
          style: isCount ? 'number' : columnLabelFormat.style
        };
        return format;
      }
      return DEFAULT_COLUMN_LABEL_FORMAT;
    }, [metricHeader, headerColumnLabelFormat]);

    const subHeaderColumnLabelFormat: ColumnLabelFormat = useMemo(() => {
      const isDerivedSubTitle = typeof metricSubHeader === 'object' && metricSubHeader?.columnId;
      if (isDerivedSubTitle) {
        return columnLabelFormats[metricSubHeader.columnId];
      }
      return DEFAULT_COLUMN_LABEL_FORMAT;
    }, [metricSubHeader, columnLabelFormats]);

    const subHeaderFormat: ColumnLabelFormat = useMemo(() => {
      const isDerivedSubTitle = typeof metricSubHeader === 'object' && metricSubHeader?.columnId;
      if (isDerivedSubTitle) {
        const columnLabelFormat = subHeaderColumnLabelFormat;
        const isCount = metricValueAggregate === 'count' && columnLabelFormat.style !== 'date';
        const format: ColumnLabelFormat = {
          ...columnLabelFormat,
          style: isCount ? 'number' : columnLabelFormat.style
        };
        return format;
      }
      return DEFAULT_COLUMN_LABEL_FORMAT;
    }, [metricSubHeader, subHeaderColumnLabelFormat]);

    const formattedHeader = useMemo(() => {
      if (!metricHeader) return '';
      const isStringTitle = typeof metricHeader === 'string';
      if (isStringTitle) return metricHeader;

      const { useValue, columnId } = metricHeader;
      if (useValue) {
        const fallbackAggregateValue = fallbackAggregate(
          metricHeader.columnId,
          metricHeader.aggregate,
          columnLabelFormats
        );

        const operator = new JsonDataFrameOperationsSingle(data, columnId);
        const value = operator[fallbackAggregateValue]();
        return formatLabel(value, headerLabelFormat, false);
      }
      return formatLabel(metricHeader.columnId, headerLabelFormat, true);
    }, [metricHeader, firstRow, headerLabelFormat]);

    const formattedSubHeader = useMemo(() => {
      if (!metricSubHeader) return '';
      const isStringTitle = typeof metricSubHeader === 'string';
      if (isStringTitle) return metricSubHeader;

      const { useValue, columnId } = metricSubHeader;
      if (useValue) {
        const fallbackAggregateValue = fallbackAggregate(
          metricSubHeader.columnId,
          metricSubHeader.aggregate,
          columnLabelFormats
        );
        const operator = new JsonDataFrameOperationsSingle(data, columnId);
        const value = operator[fallbackAggregateValue]();
        return formatLabel(value, subHeaderFormat, false);
      }
      return formatLabel(metricSubHeader.columnId, subHeaderFormat, true);
    }, [metricSubHeader, firstRow, subHeaderFormat]);

    const formattedValue = useMemo(() => {
      if (metricValueAggregate && !metricValueLabel) {
        const operator = new JsonDataFrameOperationsSingle(data, metricColumnId);
        const isCount = metricValueAggregate === 'count';
        const format: ColumnLabelFormat = {
          ...yLabelFormat,
          style: isCount ? 'number' : yLabelFormat.style
        };

        return formatLabel(operator[metricValueAggregate](), format);
      } else if (metricValueLabel) {
        return metricValueLabel;
      }

      return formatLabel(firstRowValue, yLabelFormat);
    }, [firstRowValue, metricValueAggregate, yLabelFormat]);

    const memoizedAnimation = useMemo(() => {
      if (!animate) return {};

      return {
        initial: { opacity: 0 },
        animate: { opacity: 1 },
        exit: { opacity: 0 },
        transition: { duration: 0.6 }
      };
    }, [animate]);

    useMount(async () => {
      requestAnimationFrame(() => {
        onMounted?.();
      });
      timeout((memoizedAnimation?.transition?.duration || 0.01) * 1000).then(() => {
        onInitialAnimationEnd?.();
      });
    });

    return (
      <AnimatePresence>
        <motion.div
          className={`flex h-full w-full flex-col items-center justify-center ${className}`}
          {...memoizedAnimation}>
          <AnimatedTitleWrapper title={formattedHeader} type="header" />
          <div className="py-1.5">
            <Title {...titleProps}>{formattedValue}</Title>
          </div>
          <AnimatedTitleWrapper title={formattedSubHeader} type="subHeader" />
        </motion.div>
      </AnimatePresence>
    );
  }
);
BusterMetricChart.displayName = 'BusterMetricChart';

const titleProps = {
  ellipsis: {
    tooltip: true
  }
};

const AnimatedTitleWrapper = ({ title, type }: { title: string; type: 'header' | 'subHeader' }) => {
  const memoizedAnimation = useMemo(() => {
    return {
      initial: {
        opacity: 0,
        height: 0,
        scale: 0.95,
        y: type === 'header' ? -4 : 4
      },
      animate: {
        opacity: 1,
        height: 'auto',
        scale: 1,
        y: 0
      },
      exit: {
        opacity: 0,
        height: 0,
        scale: 0.94,
        y: type === 'header' ? -7 : 4
      },
      transition: {
        duration: 0.25,
        ease: [0.4, 0, 0.2, 1],
        height: {
          duration: 0.2
        },
        opacity: {
          duration: 0.25,
          delay: 0.05
        },
        scale: {
          duration: 0.25
        }
      }
    };
  }, []);

  return (
    <AnimatePresence mode="wait" initial={false}>
      {title && (
        <motion.div className="overflow-visible" {...memoizedAnimation}>
          <motion.div className="origin-center">
            <Title {...titleProps} level={4}>
              {title}
            </Title>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
};

const fallbackAggregate = (
  columnId: string,
  aggregate: MetricChartProps['metricValueAggregate'] = 'sum',
  columnLabelFormats: BusterMetricChartProps['columnLabelFormats']
): NonNullable<MetricChartProps['metricValueAggregate']> => {
  const columnLabelFormat = columnLabelFormats[columnId];
  const isNumber =
    columnLabelFormat.style === 'number' && columnLabelFormat.columnType === 'number';
  const isValid = isNumber;
  if (isValid) return aggregate;
  return 'first';
};

AnimatedTitleWrapper.displayName = 'AnimatedTitleWrapper';
