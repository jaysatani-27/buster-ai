import { Button, ButtonProps } from 'antd';
import { createStyles } from 'antd-style';
import React, { PropsWithChildren } from 'react';

const useStyles = createStyles(({ css, token }) => ({
  selected: {
    background: token.colorBgContainerDisabled
  }
}));

export const AppButtonSelect: React.FC<
  PropsWithChildren<
    ButtonProps & {
      selected?: boolean;
    }
  >
> = ({ children, selected, ...props }) => {
  const { cx, styles } = useStyles();

  return (
    <Button {...props} className={cx(props.className, selected && styles.selected)}>
      {children}
    </Button>
  );
};
