import { useStyles } from './useStyles';
import React from 'react';

export const TooltipTitle: React.FC<{ title: string }> = ({ title }) => {
  const { styles, cx } = useStyles();
  return (
    <div className={cx(styles.tooltipTitleContainer, 'px-3 py-1.5')}>
      <span className="title">{title}</span>
    </div>
  );
};
