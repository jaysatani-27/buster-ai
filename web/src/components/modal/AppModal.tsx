'use client';

import { createStyles } from 'antd-style';
import React, { PropsWithChildren, useEffect } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { createPortal } from 'react-dom';
import { Button, ButtonProps } from 'antd';
import { AppMaterialIcons } from '../icons';
import { Title, Text } from '../text';

const useStyles = createStyles(({ token, css }) => ({
  modalOverlay: css`
    background-color: ${token.colorBgMask};
    position: fixed;
    inset: 0;
    z-index: 50;
  `,
  modalContainerWrapper: css`
    position: fixed;
    top: 300px;
    left: 50%;
    transform: translate(-50%, -50%);
    z-index: 51;
  `,
  modalContainer: css`
    background-color: white;
    border-radius: ${token.borderRadius}px;
    box-shadow: ${token.boxShadow};
    min-width: 300px;
    max-width: 90vw;
    max-height: 90vh;
    overflow-y: auto;
    border: 0.5px solid ${token.colorBorder};
  `,
  modalContent: css`
    padding: 24px 32px 0px 32px;
  `,

  modalBody: css`
    padding-bottom: 16px;
  `,
  modalHeader: css`
    padding: 0 0 12px 0;
  `,
  modalFooter: css`
    padding: 12px 32px;
    border-top: 0.5px solid ${token.colorBorder};
  `,
  modalCloseButton: css`
    position: absolute;
    top: 32px;
    right: 32px;
    color: ${token.colorIcon};
    cursor: pointer;
    transform: translate(0%, -20%);
    &:hover {
      color: ${token.colorIconHover};
    }
  `
}));

export interface AppModalProps {
  className?: string;
  style?: React.CSSProperties;
  open: boolean;
  onClose: () => void;
  footer: {
    left?: React.ReactNode;
    primaryButton: {
      text: string;
      onClick: () => void;
      type?: ButtonProps['type'];
      loading?: boolean;
      disabled?: boolean;
    };
    secondaryButton?: {
      text: string;
      onClick: () => void;
      type?: ButtonProps['type'];
      loading?: boolean;
      disabled?: boolean;
    };
  };
  header: {
    title: string;
    description?: string;
  };
  width?: number;
}

export const AppModal: React.FC<PropsWithChildren<AppModalProps>> = React.memo(
  ({ open, style, className = '', onClose, children, header, footer, width = 600 }) => {
    const { styles, cx } = useStyles();
    const [isMounted, setIsMounted] = React.useState(false);

    useEffect(() => {
      setIsMounted(true);
    }, []);

    if (!isMounted) {
      return null;
    }

    return createPortal(
      <AnimatePresence>
        {open && (
          <>
            <motion.div
              className={styles.modalOverlay}
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0, transition: { delay: 0.08 } }}
              onClick={onClose}
              transition={{ duration: 0.12, ease: 'easeOut' }}
            />

            <div className={cx(styles.modalContainerWrapper, 'relative')}>
              <motion.div
                className={cx(styles.modalContainer)}
                initial={{ opacity: 0, scale: 0.95 }}
                animate={{ opacity: 1, scale: 1 }}
                exit={{ opacity: 0, scale: 0.95 }}
                transition={{
                  duration: 0.2,
                  ease: [0.33, 1, 0.68, 1]
                }}
                style={{
                  width: `${width}px`
                }}>
                <div className={cx(styles.modalContent)}>
                  <ModalHeader header={header} />
                  <div className={styles.modalBody}>{children}</div>
                </div>

                {footer && <ModalFooter footer={footer} />}

                <div className={styles.modalCloseButton} onClick={onClose}>
                  <AppMaterialIcons size={16} icon="close" />
                </div>
              </motion.div>
            </div>
          </>
        )}
      </AnimatePresence>,
      document.body
    );
  }
);

AppModal.displayName = 'AppModal';
const ModalHeader: React.FC<PropsWithChildren<{ header: AppModalProps['header'] }>> = ({
  header
}) => {
  const { styles, cx } = useStyles();
  return (
    <div className={cx(styles.modalHeader, 'flex flex-col gap-2')}>
      <Title ellipsis className="w-[calc(100%_-_20px)]" level={3}>
        {header.title}
      </Title>
      {header.description && (
        <Text type="secondary" size="md" ellipsis={false} lineHeight={20}>
          {header.description}
        </Text>
      )}
    </div>
  );
};

const ModalFooter: React.FC<
  PropsWithChildren<{ footer: NonNullable<AppModalProps['footer']> }>
> = ({ footer }) => {
  const { styles, cx } = useStyles();
  return (
    <div
      className={cx(styles.modalFooter, 'flex', footer.left ? 'justify-between' : 'justify-end')}>
      {footer.left && <div className="flex-1">{footer.left}</div>}
      <div className="flex gap-2">
        {footer.secondaryButton && (
          <div className="flex-1">
            <Button
              type={footer.secondaryButton.type ?? 'text'}
              loading={footer.secondaryButton.loading}
              disabled={footer.secondaryButton.disabled}
              onClick={footer.secondaryButton.onClick}>
              {footer.secondaryButton.text}
            </Button>
          </div>
        )}

        {footer.primaryButton && (
          <div className="flex-1">
            <Button
              loading={footer.primaryButton.loading}
              disabled={footer.primaryButton.disabled}
              type={footer.primaryButton.type ?? 'default'}
              onClick={footer.primaryButton.onClick}>
              {footer.primaryButton.text}
            </Button>
          </div>
        )}
      </div>
    </div>
  );
};
