import React from 'react';
import { AppMaterialIcons } from '../icons';
import { createStyles } from 'antd-style';

const useStyles = createStyles(({ token, css }) => ({
  expandIcon: css`
    font-size: 10px;
    color: ${token.colorIcon};
    margin-left: 4px;
  `
}));

export const ExpandIcon: React.FC<{ isOpen?: boolean; isActive?: boolean }> = ({
  isOpen,
  isActive
}) => {
  const open = isOpen || isActive;
  const { styles, cx } = useStyles();

  return (
    <AppMaterialIcons
      icon="arrow_drop_down"
      size={18}
      className={cx('expand-icon transition', styles.expandIcon)}
      style={{
        transform: !open ? 'rotate(-90deg)' : 'rotate(0deg)'
      }}
    />
  );
};
