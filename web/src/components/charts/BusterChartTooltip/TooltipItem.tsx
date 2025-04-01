import React from 'react';
import { useStyles } from './useStyles';
import type { ITooltipItem, TooltipItemValueProps } from './interfaces';
import { LegendItemDot } from '../BusterChartLegend';
import { ChartType } from '../interfaces';

export const TooltipItem: React.FC<ITooltipItem> = ({
  values,
  color,
  seriesType,
  formattedLabel,
  usePercentage
}) => {
  const { styles, cx } = useStyles();
  const isScatter = seriesType === 'scatter';

  return (
    <>
      {formattedLabel && (
        <>
          <div className="flex items-center space-x-1.5 overflow-hidden pl-3 pr-3">
            <LegendItemDot color={color} type={seriesType as ChartType} inactive={false} />
            <span
              className={cx(styles.tooltipItemLabel, 'truncate', {
                title: isScatter
              })}>
              {formattedLabel}
            </span>
          </div>

          {isScatter && <div className={cx(styles.tooltipItemSeparator)} />}
        </>
      )}

      <TooltipItemValue values={values} usePercentage={usePercentage} isScatter={isScatter} />
    </>
  );
};

const TooltipItemValue: React.FC<{
  values: TooltipItemValueProps[];
  usePercentage: boolean;
  isScatter: boolean;
}> = ({ values, usePercentage, isScatter }) => {
  const { styles, cx } = useStyles();

  const chooseValue = (
    value: string | number | undefined,
    percentage: string | number | undefined
  ) => {
    return usePercentage ? percentage : value;
  };

  if (values.length > 1 || isScatter) {
    return (
      <div className="grid grid-cols-[auto_auto] items-center gap-x-5 px-3">
        {values.map((value, index) => (
          <GroupTooltipValue
            key={index}
            label={value.formattedLabel}
            value={chooseValue(value.formattedValue, value.formattedPercentage)}
          />
        ))}
      </div>
    );
  }

  const formattedValue = values[0]?.formattedValue;
  return (
    <div className={cx(styles.tooltipItemValue, 'tooltip-values px-3')}>
      {chooseValue(formattedValue, values[0]?.formattedPercentage)}
    </div>
  );
};

const GroupTooltipValue: React.FC<{
  label: string;
  value: string | number | undefined;
}> = ({ label, value }) => {
  const { styles, cx } = useStyles();

  return (
    <>
      <div className={cx(styles.tooltipItemLabel, 'truncate')}>{label}</div>
      <div className={cx(styles.tooltipItemValue)}>{value}</div>
    </>
  );
};
