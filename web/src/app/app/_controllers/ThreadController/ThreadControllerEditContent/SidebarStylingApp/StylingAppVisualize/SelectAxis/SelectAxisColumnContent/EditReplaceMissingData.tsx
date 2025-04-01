import React, { useMemo } from 'react';
import { LabelAndInput } from '../../../Common';
import { Select } from 'antd';
import { IColumnLabelFormat } from '@/components/charts';

export const MISSING_VALUES_OPTIONS: {
  label: string;
  value: IColumnLabelFormat['replaceMissingDataWith'];
}[] = [
  { label: 'Zero', value: 0 },
  { label: 'Do not replace', value: '🧸✂️' }
];

export const EditReplaceMissingData: React.FC<{
  replaceMissingDataWith: IColumnLabelFormat['replaceMissingDataWith'];
  onUpdateColumnConfig: (columnLabelFormat: Partial<IColumnLabelFormat>) => void;
}> = React.memo(
  ({ replaceMissingDataWith, onUpdateColumnConfig }) => {
    const selectedValue = useMemo(() => {
      if (replaceMissingDataWith === null) return '🧸✂️';
      return 0;
    }, [replaceMissingDataWith]);

    return (
      <LabelAndInput label="Missing values">
        <Select
          options={MISSING_VALUES_OPTIONS}
          defaultValue={selectedValue}
          onChange={(v) => {
            let value: IColumnLabelFormat['replaceMissingDataWith'];
            if (v === '🧸✂️') value = null;
            else value = 0;
            onUpdateColumnConfig({ replaceMissingDataWith: value });
          }}
        />
      </LabelAndInput>
    );
  },
  () => true
);

EditReplaceMissingData.displayName = 'EditReplaceMissingData';
