import React from 'react';
import { LabelAndInput } from '../../../Common/LabelAndInput';
import type { IColumnLabelFormat } from '@/components/charts/interfaces/columnLabelInterfaces';
import { Input } from 'antd';

export const EditSuffix: React.FC<{
  suffix: IColumnLabelFormat['suffix'];
  onUpdateColumnConfig: (columnLabelFormat: Partial<IColumnLabelFormat>) => void;
}> = React.memo(
  ({ suffix, onUpdateColumnConfig }) => {
    return (
      <LabelAndInput label="Suffix">
        <Input
          className="!w-full"
          min={0}
          defaultValue={suffix}
          placeholder="dollars"
          onChange={(e) =>
            onUpdateColumnConfig({
              suffix: e.target.value ?? ''
            })
          }
        />
      </LabelAndInput>
    );
  },
  () => {
    return true;
  }
);
EditSuffix.displayName = 'EditSuffix';
