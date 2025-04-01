import { createStyles } from 'antd-style';
import React from 'react';

const useStyles = createStyles(({ token }) => {
  return {
    ringContainer: {
      position: 'relative'
    },
    ringring: {
      borderRadius: '50%',
      border: `2px solid ${token.colorPrimary}`,
      position: 'absolute',
      animation: 'pulsate 1s ease-out',
      animationIterationCount: 'infinite',
      opacity: 0.0,
      height: '12px',
      width: '12px',
      left: '-6px',
      top: '-6px',
      transform: 'translate(-50%, -50%)'
    },
    circle: {
      borderRadius: '50%',
      backgroundColor: token.colorPrimary,
      position: 'absolute',
      top: 0,
      left: 0,
      transform: 'translate(-50%, -50%)'
    }
  };
});

export const PulsingDot: React.FC<{
  size?: number;
  style?: React.CSSProperties;
  color?: string;
}> = ({ style, size = 7, color }) => {
  const { cx, styles } = useStyles();

  return (
    <>
      <span className={cx('pulsing-dot relative', styles.ringContainer)} style={style}>
        <span
          className={cx(styles.ringring)}
          style={{
            // height: size * 1.35,
            // width: size * 1.35,
            // top: -(size * 1.5155) / 2,
            // left: -(size * 1.5475) / 2,
            borderColor: color
          }}
        />
        <span
          className={cx(styles.circle)}
          style={{
            width: size,
            height: size,
            backgroundColor: color
          }}
        />
      </span>
    </>
  );
};
