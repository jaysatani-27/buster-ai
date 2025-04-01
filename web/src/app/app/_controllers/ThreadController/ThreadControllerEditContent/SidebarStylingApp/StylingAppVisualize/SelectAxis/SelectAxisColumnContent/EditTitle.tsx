import { IColumnLabelFormat } from '@/components/charts';
import React from 'react';
import { LabelAndInput } from '../../../Common/LabelAndInput';
import { Input } from 'antd';

export const EditTitle: React.FC<{
  displayName: IColumnLabelFormat['displayName'];
  formattedTitle: string;
  onUpdateColumnConfig: (columnLabelFormat: Partial<IColumnLabelFormat>) => void;
}> = React.memo(
  ({ displayName, formattedTitle, onUpdateColumnConfig }) => {
    return (
      <LabelAndInput label="Title">
        <Input
          className="w-full"
          placeholder={formattedTitle}
          defaultValue={displayName || ''}
          onChange={(e) => {
            onUpdateColumnConfig({
              displayName: e.target.value
            });
          }}
        />
      </LabelAndInput>
    );
  },
  (prevProps, nextProps) => {
    return prevProps.formattedTitle === nextProps.formattedTitle;
  }
);
EditTitle.displayName = 'EditTitle';
