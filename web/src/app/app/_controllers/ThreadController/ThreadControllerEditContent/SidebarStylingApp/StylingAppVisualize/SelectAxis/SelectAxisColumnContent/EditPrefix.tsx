import React from 'react';
import { LabelAndInput } from '../../../Common/LabelAndInput';
import type { IColumnLabelFormat } from '@/components/charts/interfaces/columnLabelInterfaces';
import { Input } from 'antd';

export const EditPrefix: React.FC<{
  prefix: IColumnLabelFormat['prefix'];
  onUpdateColumnConfig: (columnLabelFormat: Partial<IColumnLabelFormat>) => void;
}> = React.memo(
  ({ prefix, onUpdateColumnConfig }) => {
    return (
      <LabelAndInput label="Prefix">
        <Input
          className="!w-full"
          min={0}
          placeholder="$"
          defaultValue={prefix}
          onChange={(e) =>
            onUpdateColumnConfig({
              prefix: e.target.value ?? ''
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
EditPrefix.displayName = 'EditPrefix';
