import { createStyles } from 'antd-style';
import React from 'react';
import { AnimatePresence, motion } from 'framer-motion';

export const BusterEChartWrapper: React.FC<{
  children: React.ReactNode;
  xAxisTitle: string;
  yAxisTitle: string;
  y2AxisTitle: string;
}> = React.memo(({ children, xAxisTitle, yAxisTitle, y2AxisTitle }) => {
  const [mounted, setMounted] = React.useState(false);

  React.useEffect(() => {
    requestAnimationFrame(() => {
      setMounted(true); //TODO. There is a bug with echarts where the chart did not get width dimensions until the next render.
    });
  }, []);

  return (
    <div className="echart-wrapper flex h-full w-full overflow-hidden">
      <YAxisTitle mounted={mounted} yAxisTitle={yAxisTitle} />
      <div className="flex h-full w-full flex-col overflow-hidden">
        <div className="h-full min-h-[100px] w-full min-w-[100px] overflow-hidden">
          {mounted ? children : null}
        </div>
        <XAxisTitle mounted={mounted} xAxisTitle={xAxisTitle} />
      </div>
      <Y2AxisTitle mounted={mounted} y2AxisTitle={y2AxisTitle} />
    </div>
  );
});
BusterEChartWrapper.displayName = 'BusterEChartWrapper';

const xAxisAnimation = {
  initial: { opacity: 0, height: 0 },
  animate: { opacity: 1, height: 'auto' },
  exit: { opacity: 0, height: 0 },
  transition: {
    duration: 0.2,
    opacity: { duration: 0.12 }
  }
};

const XAxisTitle: React.FC<{ xAxisTitle: string; mounted: boolean }> = React.memo(
  ({ xAxisTitle, mounted }) => {
    const { styles, cx } = useStyles();

    return (
      <AnimatePresence initial={false}>
        {xAxisTitle && (
          <motion.div
            {...xAxisAnimation}
            className={cx('ml-[28px] flex items-center justify-center', !mounted && 'hidden')}>
            <div className={cx(styles.title, 'px-3.5')}>{xAxisTitle}</div>
          </motion.div>
        )}
      </AnimatePresence>
    );
  }
);
XAxisTitle.displayName = 'XAxisTitle';

const yAxisAnimation = {
  initial: { opacity: 0, width: 0 },
  animate: { opacity: 1, width: 'auto' },
  exit: { opacity: 0, width: 0 },
  transition: {
    duration: 0.2,
    opacity: { duration: 0.12 }
  }
};

const YAxisTitle: React.FC<{ yAxisTitle: string; mounted: boolean }> = React.memo(
  ({ yAxisTitle, mounted }) => {
    const { styles, cx } = useStyles();
    const showAxis = !!yAxisTitle;

    return (
      <AnimatePresence initial={false}>
        {showAxis && (
          <motion.div
            className={cx(
              styles.yTitle,
              'flex items-center justify-center py-3.5',
              !mounted && 'hidden'
            )}
            {...yAxisAnimation}>
            <div className={styles.title}>{yAxisTitle}</div>
          </motion.div>
        )}
      </AnimatePresence>
    );
  }
);
YAxisTitle.displayName = 'YAxisTitle';

const Y2AxisTitle: React.FC<{ y2AxisTitle: string; mounted: boolean }> = React.memo(
  ({ y2AxisTitle, mounted }) => {
    const { styles, cx } = useStyles();
    return (
      <AnimatePresence initial={false}>
        {y2AxisTitle && (
          <motion.div
            {...yAxisAnimation}
            className={cx(
              styles.y2Title,
              'flex items-center justify-center py-3.5 pl-1',
              !mounted && 'hidden'
            )}>
            <div className={styles.title}>{y2AxisTitle}</div>
          </motion.div>
        )}
      </AnimatePresence>
    );
  }
);
Y2AxisTitle.displayName = 'Y2AxisTitle';

const useStyles = createStyles(({ css, token }) => ({
  title: css`
    font-size: 12px;
    color: ${token.colorTextSecondary};
    user-select: none;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  `,
  yTitle: css`
    writing-mode: vertical-rl;
    transform: rotate(180deg);
  `,
  y2Title: css`
    writing-mode: vertical-rl;
    transform: rotate(0deg);
  `
}));
