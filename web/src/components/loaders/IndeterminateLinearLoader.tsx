import { createStyles } from 'antd-style';
import React from 'react';

export const IndeterminateLinearLoader: React.FC<{
  className?: string;
  style?: React.CSSProperties;
  height?: number;
  trackColor?: string;
  valueColor?: string;
}> = ({ className = '', trackColor, valueColor, style, height = 2 }) => {
  const { styles, cx } = useStyles();

  return (
    <div
      className={`indeterminate-progress-bar ${className}`}
      style={{ ...style, height, backgroundColor: trackColor }}>
      <div
        className={cx(styles.track, 'indeterminate-progress-bar-value')}
        style={{
          backgroundColor: valueColor
        }}></div>
    </div>
  );
};

const useStyles = createStyles(({ css, token }) => ({
  track: css`
    background: ${token.colorPrimary};
    opacity: 0.6;
  `
}));
