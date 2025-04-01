import React from 'react';
import { LabelAndInput } from '../Common';
import { Switch } from 'antd';

export const EditShowDataLabels: React.FC<{
  showDataLabels: boolean;
  onUpdateColumnSettingConfig: (v: boolean) => void;
}> = React.memo(
  ({ showDataLabels, onUpdateColumnSettingConfig }) => {
    return (
      <LabelAndInput label={'Data labels'}>
        <div className="flex justify-end">
          <Switch
            defaultChecked={showDataLabels}
            onChange={(v) => onUpdateColumnSettingConfig(v)}
          />
        </div>
      </LabelAndInput>
    );
  },
  () => {
    return true;
  }
);
EditShowDataLabels.displayName = 'EditShowDataLabels';
