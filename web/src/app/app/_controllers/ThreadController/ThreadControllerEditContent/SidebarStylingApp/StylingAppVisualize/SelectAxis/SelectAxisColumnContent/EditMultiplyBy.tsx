import React, { useState } from 'react';
import { LabelAndInput } from '../../../Common/LabelAndInput';
import type { IColumnLabelFormat } from '@/components/charts/interfaces/columnLabelInterfaces';
import { InputNumber } from 'antd';

export const EditMultiplyBy: React.FC<{
  multiplier: IColumnLabelFormat['multiplier'];
  onUpdateColumnConfig: (columnLabelFormat: Partial<IColumnLabelFormat>) => void;
}> = React.memo(
  ({ multiplier, onUpdateColumnConfig }) => {
    return (
      <LabelAndInput label="Multiply By">
        <InputNumber
          className="!w-full"
          min={0}
          defaultValue={multiplier}
          onChange={(value) =>
            onUpdateColumnConfig({
              multiplier: value ?? 1
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
EditMultiplyBy.displayName = 'EditMultiplyBy';
