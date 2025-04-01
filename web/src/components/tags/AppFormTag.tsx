import { Tag } from 'antd';
import { createStyles } from 'antd-style';
import React from 'react';
import tailwind from '../../../tailwind.config';
const sizes = tailwind.theme.fontSize;

const useStyles = createStyles(({ css, token }) => {
  return {
    tag: css`
      display: flex;
      align-items: center;
      justify-content: center;
      border: 0.5px solid ${token.colorBorder};
      border-radius: ${token.borderRadius}px;
      height: ${token.controlHeight}px;
      padding: 0px 8px;
      color: ${token.colorTextTertiary};
      font-size: ${sizes.sm};
    `
  };
});

export const AppFormatTag: React.FC<{
  onClick?: () => void;
  children: string | React.ReactNode;
  disabled?: boolean;
  className?: string;
}> = ({ children, className = '', onClick, disabled }) => {
  const { cx, styles } = useStyles();

  return (
    <>
      <div
        className={cx(className, styles.tag, disabled ? 'cursor-not-allowed' : 'cursor-pointer')}
        onClick={onClick}>
        {children}
      </div>
    </>
  );
};
