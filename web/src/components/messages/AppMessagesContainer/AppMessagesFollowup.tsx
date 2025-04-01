import React, { useLayoutEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { createStyles } from 'antd-style';
import { inputHasText } from '@/utils/text';
import { Input, Button } from 'antd';
import { AppMaterialIcons } from '@/components/icons';
import { useMemoizedFn } from 'ahooks';

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
      &.disabled {
        border-color: ${token.colorBorder} !important;
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

const INPUT_STYLE = {
  paddingRight: 38
};
const AUTO_SIZE = { minRows: 1, maxRows: 10 };
const CONTAINER_MOTION = {
  initial: { opacity: 0, y: 0 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: 0 },
  transition: { delay: 0.1 }
};
const BUTTON_MOTION = {
  initial: { opacity: 0 },
  animate: { opacity: 1 },
  exit: { opacity: 0 },
  transition: { delay: 0.285 }
};

export const AppMessagesFollowup: React.FC<{
  showSkeletonLoader: boolean;
  defaultValue?: string;
  onSend: (value: string) => Promise<void>;
  onChange?: (value: string) => void;
  placeholder: string;
  className?: string;
  disabled?: boolean;
}> = React.memo(
  ({
    showSkeletonLoader,
    defaultValue,
    onChange,
    onSend,
    placeholder,
    className = '',
    disabled
  }) => {
    const { cx, styles } = useStyles();

    const [submitting, setSubmitting] = useState(false);
    const [value, setValue] = useState(defaultValue || '');
    const [isFocused, setIsFocused] = useState(false);

    const disableSubmit = !inputHasText(value) || disabled;
    const disableInput = disabled;

    const onSendMessage = useMemoizedFn(async () => {
      setSubmitting(true);
      try {
        await onSend(value);
        setValue('');
      } catch (error) {
        //
      }
      setSubmitting(false);
    });

    const onChangeTextInput = useMemoizedFn((e: React.ChangeEvent<HTMLTextAreaElement>) => {
      setValue(e.target.value);
      onChange?.(e.target.value);
    });

    const onBlurTextInput = useMemoizedFn(() => {
      setIsFocused(false);
    });

    const onFocusTextInput = useMemoizedFn(() => {
      setIsFocused(true);
    });

    useLayoutEffect(() => {
      setValue(defaultValue || '');
    }, [defaultValue]);

    return (
      <motion.div
        initial={CONTAINER_MOTION.initial}
        animate={CONTAINER_MOTION.animate}
        exit={CONTAINER_MOTION.exit}
        transition={CONTAINER_MOTION.transition}
        className={cx(styles.textContainer, className, 'relative p-4')}>
        <div className="relative">
          {showSkeletonLoader ? (
            <div className={cx(styles.skeletonLoader)}></div>
          ) : (
            <div
              className={cx(
                styles.inputContainer,
                isFocused && 'focused',
                'relative flex items-center',
                disableInput && 'disabled'
              )}>
              <Input.TextArea
                variant="borderless"
                onBlur={onBlurTextInput}
                onFocus={onFocusTextInput}
                className="inline-block !pl-3.5 align-middle"
                placeholder={placeholder}
                value={value}
                autoFocus={true}
                style={INPUT_STYLE}
                onChange={onChangeTextInput}
                onPressEnter={onSendMessage}
                autoSize={AUTO_SIZE}
                disabled={submitting || disabled}
                tabIndex={0}
              />

              <FollowupButton
                showSkeletonLoader={showSkeletonLoader}
                disableSubmit={disableSubmit}
                submitting={submitting}
                onSendMessage={onSendMessage}
              />
            </div>
          )}
        </div>
      </motion.div>
    );
  }
);
AppMessagesFollowup.displayName = 'AppMessagesFollowup';

const FollowupButton = React.memo(
  ({
    showSkeletonLoader,
    disableSubmit,
    submitting,
    onSendMessage
  }: {
    showSkeletonLoader: boolean;
    disableSubmit?: boolean;
    submitting: boolean;
    onSendMessage: () => Promise<void>;
  }) => {
    const { cx, styles } = useStyles();

    return (
      <motion.div
        initial={BUTTON_MOTION.initial}
        animate={BUTTON_MOTION.animate}
        exit={BUTTON_MOTION.exit}
        transition={BUTTON_MOTION.transition}
        className="absolute right-0 top-0 flex h-full -translate-x-1/2 flex-col items-center justify-center">
        {showSkeletonLoader ? (
          <div className={cx(styles.skeletonLoaderButton)}></div>
        ) : (
          <Button
            disabled={disableSubmit}
            loading={submitting}
            type="primary"
            onClick={onSendMessage}
            icon={<AppMaterialIcons icon="arrow_upward" size={16} />}></Button>
        )}
      </motion.div>
    );
  }
);
FollowupButton.displayName = 'FollowupButton';
