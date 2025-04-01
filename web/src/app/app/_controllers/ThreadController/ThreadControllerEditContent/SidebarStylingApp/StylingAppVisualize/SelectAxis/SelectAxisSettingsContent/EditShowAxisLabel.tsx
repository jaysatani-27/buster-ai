import React from 'react';
import { SelectAxisContainerId } from '../config';
import { LabelAndInput } from '../../../Common/LabelAndInput';
import { Switch } from 'antd';

export const EditShowAxisLabel: React.FC<{
  showAxisLabel: boolean;
  onChangeShowAxisLabel: (value: boolean) => void;
}> = React.memo(
  ({ showAxisLabel, onChangeShowAxisLabel }) => {
    return (
      <LabelAndInput label="Show axis label">
        <div className="flex justify-end">
          <Switch defaultChecked={showAxisLabel} onChange={onChangeShowAxisLabel} />
        </div>
      </LabelAndInput>
    );
  },
  (prevProps, nextProps) => {
    return true;
  }
);
EditShowAxisLabel.displayName = 'EditShowAxisLabel';
