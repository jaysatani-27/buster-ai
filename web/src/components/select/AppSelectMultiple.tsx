import { SelectProps, Select } from 'antd';
import React from 'react';
import { AppSelectItem } from './AppSelectItem';
import { useMemoizedFn } from 'ahooks';
import { createStyles } from 'antd-style';

export interface AppSelectMultipleProps extends SelectProps {
  value: string[];
  options: Required<SelectProps['options']>;
  onChange: (value: string[]) => void;
}

const useStyles = createStyles(({ token, css }) => ({
  select: css``,
  selectPopup: css`
    .busterv2-select-item-option-selected {
      .busterv2-select-item-option-state {
        display: none !important;
      }
      border-radius: ${token.borderRadius} !important;
      :not(:hover) {
        background-color: ${token.colorBgBase} !important;
      }
    }
  `
}));

export const AppSelectMultiple: React.FC<AppSelectMultipleProps> = ({ ...props }) => {
  const { styles, cx } = useStyles();
  const value = props.value || [];

  const onSelectPreflight = useMemoizedFn((id: string) => {
    const isOptionSelected = value.includes(id);
    const newValue = isOptionSelected ? value.filter((i) => i !== id) : [...value, id];
    props.onChange(newValue);
  });

  return (
    <Select
      mode="multiple"
      {...props}
      maxTagCount={'responsive'}
      className={cx(styles.select, props.className)}
      popupClassName={cx(styles.selectPopup, props.popupClassName)}
      filterOption={(input, option) =>
        (String(option?.label) ?? '').toLowerCase().includes(input.toLowerCase())
      }
      optionRender={(option) => {
        return (
          <AppSelectItem
            hideCheckmark={true}
            content={option.label}
            selected={value.includes(option.value as string)}
            onClick={() => onSelectPreflight(option.value as string)}
          />
        );
      }}
    />
  );
};
