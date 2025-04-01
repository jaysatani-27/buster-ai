import { ColumnSettings } from '@/components/charts';
import React from 'react';
import { LabelAndInput } from '../../../Common/LabelAndInput';
import { Switch } from 'antd';

export const EditShowDataLabel: React.FC<{
  showDataLabels: Required<ColumnSettings>['showDataLabels'];
  onUpdateColumnSettingConfig: (columnSettings: Partial<ColumnSettings>) => void;
}> = React.memo(
  ({ showDataLabels, onUpdateColumnSettingConfig }) => {
    return (
      <LabelAndInput label="Show data labels">
        <div className="flex justify-end">
          <Switch
            defaultChecked={showDataLabels}
            onChange={(v) => {
              onUpdateColumnSettingConfig({ showDataLabels: v });
            }}
          />
        </div>
      </LabelAndInput>
    );
  },
  () => {
    return true;
  }
);
EditShowDataLabel.displayName = 'EditShowDataLabel';
