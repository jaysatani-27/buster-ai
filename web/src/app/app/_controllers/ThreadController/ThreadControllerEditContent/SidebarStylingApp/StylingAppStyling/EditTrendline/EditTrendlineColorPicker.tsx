import { ColorPicker } from 'antd';
import React from 'react';
import { LabelAndInput } from '../../Common';
import { LoopTrendline } from './EditTrendline';
import { useMemoizedFn } from 'ahooks';
import { Color } from 'antd/es/color-picker';

export const TrendlineColorPicker = React.memo(
  ({
    trend,
    onUpdateExisitingTrendline
  }: {
    trend: LoopTrendline;
    onUpdateExisitingTrendline: (trend: LoopTrendline) => void;
  }) => {
    const onChangeComplete = useMemoizedFn((color: Color) => {
      const hexColor = color.toHexString();
      onUpdateExisitingTrendline({ ...trend, trendLineColor: hexColor });
    });

    const onClear = useMemoizedFn(() => {
      onUpdateExisitingTrendline({ ...trend, trendLineColor: null });
    });

    return (
      <LabelAndInput label="Color">
        <div className="flex w-full items-center justify-end">
          <ColorPicker
            size="small"
            defaultValue={trend.trendLineColor || 'black'}
            onChangeComplete={onChangeComplete}
            onClear={onClear}
          />
        </div>
      </LabelAndInput>
    );
  }
);
TrendlineColorPicker.displayName = 'TrendlineColorPicker';
