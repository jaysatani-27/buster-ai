'use client';

import React, { useEffect, useLayoutEffect, useRef } from 'react';
import { Input, InputRef } from 'antd';
import { createStyles } from 'antd-style';
import { useAntToken } from '@/styles/useAntToken';

const useStyles = createStyles(({ css, token }) => {
  return {
    input: css`
      line-height: ${token.lineHeightHeading4};
      color: ${token.colorTextBase} !important;
      cursor: text !important;
    `,
    level1: css`
      font-size: ${token.fontSizeHeading1}px;
    `,
    level2: css`
      font-size: ${token.fontSizeHeading2}px;
    `,
    level3: css`
      font-size: ${token.fontSizeHeading3}px;
    `,
    level4: css`
      font-size: ${token.fontSizeHeading4}px;
      .isEditing {
        //
      }
    `,
    level5: css`
      font-size: ${token.fontSizeHeading5}px;
    `
  };
});

export const EditableTitle = React.memo(
  React.forwardRef<
    HTMLDivElement,
    {
      children: string;
      extraChildren?: React.ReactNode;
      onEdit?: (b: boolean) => void;
      onChange: (value: string) => void;
      onSetValue?: (value: string) => void;
      onPressEnter?: () => void;
      level?: 1 | 2 | 3 | 4 | 5;
      disabled?: boolean;
      editing?: boolean;
      className?: string;
      placeholder?: string;
      style?: React.CSSProperties;
      showBottomBorder?: boolean;
    }
  >(
    (
      {
        showBottomBorder,
        style,
        disabled,
        className = '',
        placeholder,
        onPressEnter,
        extraChildren,
        editing,
        children,
        level = 4,
        onEdit,
        onChange,
        onSetValue
      },
      ref
    ) => {
      const token = useAntToken();
      const { cx, styles } = useStyles();
      const inputRef = useRef<InputRef>(null);
      const [value, setValue] = React.useState(children);

      useLayoutEffect(() => {
        setValue(children);
      }, [children]);
      useEffect(() => {
        if (editing) {
          inputRef.current?.focus();
          inputRef.current?.select();
        }
      }, [editing]);

      return (
        <div
          ref={ref}
          className={cx('relative flex items-center justify-between', className)}
          style={style}>
          <Input
            placeholder={placeholder}
            ref={inputRef}
            disabled={disabled}
            variant="borderless"
            className={cx('w-full !px-0 !py-0', styles.input, {
              [styles.level1]: level === 1,
              [styles.level2]: level === 2,
              [styles.level3]: level === 3,
              [styles.level4]: level === 4,
              [styles.level5]: level === 5
            })}
            value={value}
            onChange={(e) => {
              if (e.target.value !== value) setValue(e.target.value);
              onSetValue?.(e.target.value);
            }}
            onBlur={() => {
              onChange(value);
              onEdit?.(false);
            }}
            onFocus={() => {
              onEdit?.(true);
            }}
            onPressEnter={() => {
              onChange(value);
              onPressEnter?.();
            }}
          />

          {showBottomBorder && (
            <div
              className="absolute w-full"
              style={{
                bottom: -3,
                borderBottom: `0.5px solid ${token.colorPrimary}`
              }}
            />
          )}
        </div>
      );
    }
  )
);

EditableTitle.displayName = 'EditableTitle';
