import React, { useMemo } from 'react';
import { LabelAndInput } from '../../../Common/LabelAndInput';
import { Select } from 'antd';
import { IColumnLabelFormat } from '@/components/charts/interfaces';
import {
  getDefaultDateOptions,
  getDefaultDayOfWeekOptions,
  getDefaultMonthOptions,
  getDefaultQuarterOptions
} from './dateConfig';
import first from 'lodash/last';
import { formatDate, getNow } from '@/utils/date';
import { useMemoizedFn } from 'ahooks';

export const EditDateFormat: React.FC<{
  dateFormat: IColumnLabelFormat['dateFormat'];
  convertNumberTo: IColumnLabelFormat['convertNumberTo'];
  columnType: IColumnLabelFormat['columnType'];
  onUpdateColumnConfig: (columnLabelFormat: Partial<IColumnLabelFormat>) => void;
}> = React.memo(
  ({ dateFormat, columnType, convertNumberTo, onUpdateColumnConfig }) => {
    const now = useMemo(() => getNow().toDate(), []);

    const useAlternateFormats = useMemo(() => {
      return columnType === 'number' && convertNumberTo;
    }, [columnType, convertNumberTo]);

    const defaultOptions = useMemo(() => {
      if (useAlternateFormats === 'day_of_week') return getDefaultDayOfWeekOptions(now);
      if (useAlternateFormats === 'month_of_year') return getDefaultMonthOptions(now);
      if (useAlternateFormats === 'quarter') return getDefaultQuarterOptions(now);
      return getDefaultDateOptions(now);
    }, [useAlternateFormats]);

    const selectOptions = useMemo(() => {
      const dateFormatIsInDefaultOptions = defaultOptions.some(({ value }) => value === dateFormat);
      if (!dateFormat || dateFormatIsInDefaultOptions || useAlternateFormats) return defaultOptions;
      return [
        ...defaultOptions,
        {
          label: formatDate({
            date: now,
            format: dateFormat
          }),
          value: dateFormat
        }
      ].filter(({ value }) => value && value !== 'auto');
    }, [dateFormat, defaultOptions, useAlternateFormats]);

    const selectedOption = useMemo(() => {
      return selectOptions.find((option) => option.value === dateFormat) || first(selectOptions);
    }, [dateFormat, selectOptions]);

    const onChange = useMemoizedFn(({ value }: { value: string }) => {
      onUpdateColumnConfig({
        dateFormat: value as IColumnLabelFormat['dateFormat']
      });
    });

    return (
      <LabelAndInput label="Date format">
        <div className="w-full overflow-hidden">
          <Select
            key={convertNumberTo}
            className="!w-full"
            popupMatchSelectWidth={false}
            options={selectOptions}
            defaultValue={selectedOption}
            labelInValue
            onChange={onChange}
          />
        </div>
      </LabelAndInput>
    );
  },
  (prevProps, nextProps) => {
    return prevProps.convertNumberTo === nextProps.convertNumberTo;
  }
);
EditDateFormat.displayName = 'EditDateFormat';
