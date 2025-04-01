import React from 'react';
import { LabelAndInput } from '../../../Common/LabelAndInput';
import { Switch } from 'antd';
import { useMemoizedFn } from 'ahooks';
import { ColumnSettings } from '@/components/charts';

export const EditShowBarLabelAsPercentage: React.FC<{
  onUpdateColumnSettingConfig: (columnSettings: Partial<ColumnSettings>) => void;
  showDataLabelsAsPercentage: ColumnSettings['showDataLabelsAsPercentage'];
}> = React.memo(
  ({ onUpdateColumnSettingConfig, showDataLabelsAsPercentage }) => {
    const onChange = useMemoizedFn((v: boolean) => {
      onUpdateColumnSettingConfig({ showDataLabelsAsPercentage: v });
    });

    return (
      <LabelAndInput label="Show label as %">
        <div className="flex justify-end">
          <Switch defaultChecked={showDataLabelsAsPercentage} onChange={onChange} />
        </div>
      </LabelAndInput>
    );
  },
  () => {
    return true;
  }
);
EditShowBarLabelAsPercentage.displayName = 'EditShowLabelAsPercentage';
