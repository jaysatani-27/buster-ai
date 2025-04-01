import { useMemoizedFn } from 'ahooks';
import { Input } from 'antd';
import React, { ReactNode } from 'react';

interface AppSearchInputProps {
  value: string;
  onChange: (value: string) => void;
  onBlur?: (value: string) => void;
  onPressEnter?: (value: string) => void;
  onSearch: (e?: string) => void;
  enterButton?: string | ReactNode | boolean;
  loading?: boolean;
  size?: 'large' | 'middle' | 'small';
  disabled?: boolean;
  placeholder?: string;
}

export const AppSearchInput: React.FC<AppSearchInputProps> = ({
  size = 'middle',
  onChange,
  onSearch,
  ...props
}) => {
  const onBlurEvent = useMemoizedFn((e: React.FocusEvent<HTMLInputElement, Element>) => {
    props.onBlur?.(e.target.value);
  });

  const onPressEnterEvent = useMemoizedFn((e: React.KeyboardEvent<HTMLInputElement>) => {
    props.onPressEnter && props.onPressEnter(props.value);
  });

  return (
    <Input.Search
      {...props}
      onBlur={onBlurEvent}
      onPressEnter={(e) => onPressEnterEvent(e)}
      onChange={(e) => onChange(e.target.value)}
      size={size}
      onSearch={(e) => onSearch(e)}
    />
  );
};

AppSearchInput.displayName = 'AppSearchInput';
