import { Switch } from 'antd';
import React from 'react';
import { LabelAndInput } from '../../Common';
import { LoopTrendline } from './EditTrendline';
import { useMemoizedFn } from 'ahooks';

export const EditTrendlineShowLine = React.memo(
  ({
    trend,
    onUpdateExisitingTrendline
  }: {
    trend: LoopTrendline;
    onUpdateExisitingTrendline: (trend: LoopTrendline) => void;
  }) => {
    const { show } = trend;

    const onChange = useMemoizedFn((checked: boolean) => {
      onUpdateExisitingTrendline({ ...trend, show: checked });
    });

    return (
      <LabelAndInput label="Show trend line">
        <div className="flex w-full justify-end">
          <Switch defaultChecked={show} onChange={onChange} />
        </div>
      </LabelAndInput>
    );
  }
);
EditTrendlineShowLine.displayName = 'EditTrendlineShowLine';
