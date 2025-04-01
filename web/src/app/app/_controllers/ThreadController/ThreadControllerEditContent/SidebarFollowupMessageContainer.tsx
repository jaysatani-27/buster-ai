import { AppMaterialIcons } from '@/components';
import { inputHasText } from '@/utils';
import { useMemoizedFn } from 'ahooks';
import { Button, Input } from 'antd';
import { createStyles } from 'antd-style';
import { motion, AnimatePresence } from 'framer-motion';
import React from 'react';

const useStyles = createStyles(({ token, css }) => {
  return {
    textContainer: {
      background: token.colorBgContainerDisabled,
      borderTop: `0.5px solid ${token.colorBorder}`
    },
    inputContainer: css`
      background: ${token.colorBgBase};
      border-radius: ${token.borderRadius}px;
      border: 0.5px solid ${token.colorBorder};
      transition: border-color 0.2s;
      min-height: 40px;
      &:hover {
        border-color: ${token.colorPrimaryHover};
      }
      &.focused {
        border-color: ${token.colorPrimary};
      }
    `,
    skeletonLoader: css`
      height: 40px;
      background: ${token.colorBgTextHover};
      border-radius: ${token.borderRadius}px;
    `,
    skeletonLoaderButton: css`
      border-radius: 100%;
      width: 24px;
      height: 24px;
      background: rgb(210 210 210 / 25%);
    `
  };
});

const sidebarAnimation = {
  initial: { opacity: 0, y: 0 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: 0 },
  transition: {
    delay: 0.1
  }
};

const inputButtonAnimation = {
  initial: { opacity: 0 },
  animate: { opacity: 1 },
  exit: { opacity: 0 },
  transition: {
    delay: 0.285
  }
};

export const SidebarFollowUpMessageContainer: React.FC<{
  className?: string;
  value: string;
  onChange: (value: string) => void;
  placeholder: string;
  onSend: () => void;
  loading?: boolean;
  children?: React.ReactNode;
  showSkeletonLoader?: boolean;
}> = ({
  children,
  loading = false,
  showSkeletonLoader,
  className = '',
  onChange,
  onSend,
  placeholder,
  value
}) => {
  const { cx, styles } = useStyles();
  const disableSubmit = !inputHasText(value);
  const [isFocused, setIsFocused] = React.useState(false);

  const onPressEnter = useMemoizedFn(() => {
    onSend();
  });

  const onChangeInput = useMemoizedFn((e: React.ChangeEvent<HTMLTextAreaElement>) => {
    onChange(e.target.value);
  });

  const onBlurInput = useMemoizedFn(() => {
    setIsFocused(false);
  });

  const onFocusInput = useMemoizedFn(() => {
    setIsFocused(true);
  });

  return (
    <motion.div
      initial={sidebarAnimation.initial}
      animate={sidebarAnimation.animate}
      exit={sidebarAnimation.exit}
      transition={sidebarAnimation.transition}
      className={cx(
        styles.textContainer,
        className,
        'relative p-4',
        !!children && 'flex flex-col space-y-2'
      )}>
      {children}

      <div className="relative">
        {showSkeletonLoader ? (
          <div className={cx(styles.skeletonLoader)}></div>
        ) : (
          <div
            className={cx(
              styles.inputContainer,
              isFocused && 'focused',
              'relative flex items-center'
            )}>
            <Input.TextArea
              variant="borderless"
              onBlur={onBlurInput}
              onFocus={onFocusInput}
              className="inline-block !pl-3.5 !pr-9 align-middle"
              placeholder={placeholder}
              value={value}
              autoFocus={true}
              onChange={onChangeInput}
              onPressEnter={onPressEnter}
              autoSize={{ minRows: 1, maxRows: 10 }}
              disabled={loading}
            />
          </div>
        )}

        <AnimatePresence>
          <motion.div
            {...inputButtonAnimation}
            className="absolute right-0 top-0 flex h-full -translate-x-1/2 flex-col items-center justify-center">
            {showSkeletonLoader ? (
              <div className={cx(styles.skeletonLoaderButton)}></div>
            ) : (
              <Button
                {...inputButtonAnimation}
                disabled={disableSubmit}
                loading={loading}
                type="primary"
                icon={<AppMaterialIcons icon="arrow_upward" size={16} />}
              />
            )}
          </motion.div>
        </AnimatePresence>
      </div>
    </motion.div>
  );
};
