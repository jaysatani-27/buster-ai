import React, { useState } from 'react';
import { LabelAndInput } from '../../../Common/LabelAndInput';
import type { IColumnLabelFormat } from '@/components/charts/interfaces/columnLabelInterfaces';
import { InputNumber } from 'antd';
import clamp from 'lodash/clamp';

export const EditDecimals: React.FC<{
  minimumFractionDigits: IColumnLabelFormat['minimumFractionDigits'];
  maximumFractionDigits: IColumnLabelFormat['maximumFractionDigits'];
  onUpdateColumnConfig: (columnLabelFormat: Partial<IColumnLabelFormat>) => void;
}> = React.memo(
  ({ minimumFractionDigits, maximumFractionDigits, onUpdateColumnConfig }) => {
    const [min, setMin] = useState(minimumFractionDigits);
    const [max, setMax] = useState(maximumFractionDigits);

    const handleMinChange = (value: number | null) => {
      const newMin = value ?? 0;
      if (newMin > (max ?? 0)) {
        // If min exceeds max, set both to the same value
        setMin(newMin);
        setMax(newMin);
        onUpdateColumnConfig({
          minimumFractionDigits: newMin,
          maximumFractionDigits: newMin
        });
      } else {
        setMin(newMin);
        onUpdateColumnConfig({ minimumFractionDigits: newMin });
      }
    };

    const handleMaxChange = (value: number | null) => {
      const newMax = value ?? 0;
      if (newMax < (min ?? 0)) {
        // If max goes below min, set both to the same value
        setMin(newMax);
        setMax(newMax);
        onUpdateColumnConfig({
          minimumFractionDigits: newMax,
          maximumFractionDigits: newMax
        });
      } else {
        setMax(newMax);
        onUpdateColumnConfig({ maximumFractionDigits: newMax });
      }
    };

    return (
      <LabelAndInput label="Decimals">
        <div className="flex w-full items-center space-x-2">
          <InputNumber
            min={0}
            prefix="Min"
            value={min}
            className="!w-full"
            onChange={(value) => handleMinChange(value)}
          />
          <InputNumber
            prefix="Max"
            max={10}
            value={max}
            className="!w-full"
            onChange={(value) => handleMaxChange(value)}
          />
        </div>
      </LabelAndInput>
    );
  },
  () => {
    return true;
  }
);
EditDecimals.displayName = 'EditDecimals';
