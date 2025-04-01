import React, { useMemo } from 'react';
import type { IColumnLabelFormat } from '@/components/charts/interfaces/columnLabelInterfaces';
import { LabelAndInput } from '../../../Common/LabelAndInput';
import { Select } from 'antd';

const options: {
  label: string;
  value: string;
}[] = [
  {
    label: '100,000',
    value: ','
  },
  {
    label: '100000',
    value: '✂'
  }
];

export const EditSeparator: React.FC<{
  numberSeparatorStyle: IColumnLabelFormat['numberSeparatorStyle'];
  onUpdateColumnConfig: (columnLabelFormat: Partial<IColumnLabelFormat>) => void;
}> = React.memo(({ numberSeparatorStyle, onUpdateColumnConfig }) => {
  const selectedSeparator = useMemo(() => {
    if (numberSeparatorStyle === null) {
      return '✂';
    }

    return options.find((option) => option.value === numberSeparatorStyle)?.value;
  }, [numberSeparatorStyle]);

  return (
    <LabelAndInput label="Separator">
      <Select
        options={options}
        value={selectedSeparator}
        onChange={(value: string) =>
          onUpdateColumnConfig({ numberSeparatorStyle: value === '✂' ? null : (value as ',') })
        }
      />
    </LabelAndInput>
  );
});
EditSeparator.displayName = 'EditSeparator';
