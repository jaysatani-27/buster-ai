import React, { useMemo, useRef, useState } from 'react';
import { ConfigProvider, Select, Tag } from 'antd';
import type { SelectProps } from 'antd';
import type { CustomTagProps } from 'rc-select/lib/BaseSelect';
import { createStyles } from 'antd-style';
import { getPredictableItemBasedOnText } from '@/utils';
import { AppMaterialIcons } from '../icons';
import { useMemoizedFn } from 'ahooks';

interface AppSelectTagInputProps extends SelectProps {
  useTagRenderer?: boolean;
  inputType?: 'text' | 'textarea';
}

const useStyles = createStyles(({ token, css }) => ({
  container: css`
    .busterv2-select-selection-overflow-item {
      display: flex;
      span {
        display: flex;
      }
    }
  `,
  textarea: css`
    min-height: 100px;
    border: 0.5px solid ${token.colorBorder};
    border-radius: ${token.borderRadius}px;
    transition: border-color 0.2s ease;

    &.focused {
      border-color: ${token.colorPrimary};
    }

    &:hover {
      border-color: ${token.colorPrimaryHover};
    }

    .busterv2-select-selector {
      height: fit-content;
      border: 0px solid;
    }
  `
}));

export const AppSelectTagInput: React.FC<AppSelectTagInputProps> = ({
  className,
  options = [],
  useTagRenderer = true,
  inputType = 'text',
  ...props
}) => {
  const { cx, styles } = useStyles();
  const selectRef = useRef<HTMLSelectElement>(null);
  const [isFocused, setIsFocused] = useState(false);

  const onSelectClick = useMemoizedFn(() => {
    selectRef.current?.focus();
  });

  const memoizedTagRender = useMemo(() => {
    return (d: CustomTagProps) =>
      useTagRenderer
        ? ColoredTagRender({ ...d, disabled: !!props.disabled })
        : props.tagRender?.(d)!;
  }, [props.tagRender, useTagRenderer, props.disabled]);

  return (
    <ConfigProvider
      theme={{
        components: {
          Select: {
            boxShadowSecondary: '',
            fontFamily:
              '--apple-system, BlinkMacSystemFont, Segoe UI, Roboto, Helvetica Neue, Arial, sans-serif'
          }
        }
      }}>
      <Select
        {...props}
        options={options}
        ref={selectRef as any}
        onClick={onSelectClick}
        className={cx(
          styles.container,
          `buster-select-multiple app-select-tag-input w-full`,
          className,
          inputType === 'textarea' && styles.textarea,
          '!cursor-text',
          {
            focused: isFocused
          }
        )}
        mode="tags"
        suffixIcon={<></>}
        tagRender={memoizedTagRender}
        dropdownRender={(menu) => <div className="!hidden">{menu}</div>}
        onFocus={() => setIsFocused(true)}
        onBlur={() => setIsFocused(false)}
        variant={inputType === 'textarea' ? 'borderless' : undefined}
      />
    </ConfigProvider>
  );
};

const ColoredTagRender = ({
  label,
  value,
  onClose,
  disabled,
  closable = true,
  className = '',
  isMaxTag,
  ...props
}: Omit<CustomTagProps, 'isMaxTag'> & {
  className?: string;
  isMaxTag?: boolean;
}) => {
  const colors = ['magenta', 'red', 'volcano', 'green', 'cyan', 'blue', 'geekblue', 'purple'];
  const chosenColor = !disabled ? getPredictableItemBasedOnText(colors, String(value)) : 'default';
  const onPreventMouseDown = (event: React.MouseEvent<HTMLSpanElement>) => {
    event.preventDefault();
    event.stopPropagation();
  };

  return (
    <Tag
      {...props}
      className={`${className} !flex items-center`}
      color={chosenColor}
      closeIcon={<AppMaterialIcons size={10} className="!text-inherit" icon="close" />}
      onMouseDown={onPreventMouseDown}
      closable={!disabled && closable}
      onClose={onClose}
      style={{ marginRight: 2 }}>
      {label}
    </Tag>
  );
};
