import { ViewType, viewTypeOptions } from '@/components/charts';
import { Segmented, SegmentedProps } from 'antd';
import React from 'react';

const options: SegmentedProps['options'] = viewTypeOptions.map((v) => {
  return {
    value: v.value,
    label: <div>{v.label}</div>
  };
});

export const ChartControlsViewType: React.FC<{
  onChangeView: (v: ViewType) => void;
  viewType: ViewType | undefined;
  disabled?: boolean;
}> = ({ onChangeView, viewType, disabled = false }) => {
  return (
    <Segmented
      options={options}
      value={viewType}
      disabled={disabled}
      onChange={(v) => onChangeView(v as ViewType)}
    />
  );
};
