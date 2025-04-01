import React from 'react';
import { Checkbox, CheckboxChangeEvent } from 'antd';
import { useMemoizedFn } from 'ahooks';

export const MemoizedCheckbox = React.memo(
  ({
    checked,
    indeterminate,
    onChange
  }: {
    checked: boolean;
    indeterminate: boolean;
    onChange: (v: boolean) => void;
  }) => {
    const handleChange = useMemoizedFn((e: CheckboxChangeEvent) => {
      onChange?.(e.target.checked);
    });

    return <Checkbox checked={checked} indeterminate={indeterminate} onChange={handleChange} />;
  }
);
MemoizedCheckbox.displayName = 'MemoizedCheckbox';
