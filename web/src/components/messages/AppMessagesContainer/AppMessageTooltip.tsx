import React from 'react';
import { AppMessageContainerMessage } from './AppMessagesContainer';
import { createStyles } from 'antd-style';
import { motion, AnimatePresence } from 'framer-motion';
import { AppTooltip } from '@/components/tooltip';
import { useMemoizedFn } from 'ahooks';

const MOTION_CONFIG = {
  initial: { opacity: 0, translateY: 3 },
  animate: { opacity: 1, translateY: 0 },
  exit: { opacity: 0 },
  transition: { duration: 0.1 }
};

export const AppMessageTooltip = React.memo<{
  tooltipItems: AppMessageContainerMessage['tooltipItems'];
  showTooltip: boolean;
}>(({ tooltipItems, showTooltip }) => {
  const { cx, styles } = useStyles();

  const onClickTooltip = useMemoizedFn((e: React.MouseEvent<HTMLDivElement>) => {
    e.stopPropagation();
    e.preventDefault();
  });

  if (!tooltipItems || tooltipItems.length === 0) {
    return null;
  }

  return (
    <AnimatePresence mode="wait">
      {showTooltip && (
        <motion.div
          onClick={onClickTooltip}
          initial={MOTION_CONFIG.initial}
          animate={MOTION_CONFIG.animate}
          exit={MOTION_CONFIG.exit}
          transition={MOTION_CONFIG.transition}
          className={cx(
            styles.tooltip,
            'absolute right-[5px] top-[-17px] !mt-0 flex items-center justify-center space-x-0 shadow-sm'
          )}>
          {tooltipItems.map((item, index) => (
            <MessageTooltipItem key={index} {...item} />
          ))}
        </motion.div>
      )}
    </AnimatePresence>
  );
});
AppMessageTooltip.displayName = 'AppMessageTooltip';

const MessageTooltipItem = React.memo(
  ({
    tooltipText,
    shortcuts,
    icon,
    onClick,
    selected
  }: NonNullable<AppMessageContainerMessage['tooltipItems']>[number]) => {
    const { cx, styles } = useStyles();

    return (
      <AppTooltip title={tooltipText} shortcuts={shortcuts}>
        <div
          className={cx(
            'flex items-center justify-center',
            styles.tooltipItem,
            selected && 'isSelected'
          )}
          onClick={onClick}>
          {icon}
        </div>
      </AppTooltip>
    );
  }
);
MessageTooltipItem.displayName = 'MessageTooltipItem';

const useStyles = createStyles(({ token, css }) => ({
  tooltip: css`
    border-radius: ${token.borderRadius}px;
    border: 0.5px solid ${token.colorBorder};
    padding: 2px 0px;
    height: 28px;
    background: ${token.colorBgBase};
    overflow: hidden;
  `,
  tooltipItem: css`
    color: ${token.colorIcon};
    height: ${token.controlHeight}px;
    width: ${token.controlHeight}px;
    display: flex;
    align-items: center;
    border: 0.5px solid transparent;
    border-radius: ${token.borderRadius - 1}px;
    transition: all 0.075s;
    &:hover {
      background: ${token.controlItemBgHover};
      color: ${token.colorText};
    }
    &.isSelected {
      background: ${token.controlItemBgActive};
      color: ${token.colorText};
    }
  `
}));
