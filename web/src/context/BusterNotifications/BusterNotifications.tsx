import React, { PropsWithChildren } from 'react';
import { App, ModalFuncProps } from 'antd';
import { createStyles } from 'antd-style';
import { useMemoizedFn } from 'ahooks';
import {
  useContextSelector,
  createContext,
  ContextSelector
} from '@fluentui/react-context-selector';

export type NotificationType = 'success' | 'info' | 'warning' | 'error';

export interface NotificationProps {
  type?: NotificationType;
  title?: string;
  message: string;
  closeIcon?: React.ReactNode | boolean;
  duration?: number;
}

const useStyles = createStyles(({ token, css }) => ({
  modal: css`
    .busterv2-modal-body {
      padding: 0px !important;
    }

    .busterv2-modal-confirm-body {
      padding: 24px 32px 16px 32px !important;
    }

    .busterv2-modal-confirm-btns {
      margin-top: 0px !important;
      padding: 12px 32px !important;
      border-top: 0.5px solid ${token.colorBorder};
      display: flex;
      align-items: center;
      justify-content: flex-end;
    }

    .busterv2-modal-confirm-content {
      color: ${token.colorTextSecondary} !important;
    }
  `
}));

export const useBusterNotificationsInternal = () => {
  const { message, notification, modal } = App.useApp();

  const { cx, styles } = useStyles();

  const openNotification = useMemoizedFn(
    (props: { title?: string; message: string; type: NotificationType }) => {
      notification?.open?.({ ...props, description: props.message, message: props.title });
    }
  );

  const openErrorNotification = useMemoizedFn((data: NotificationProps | unknown) => {
    const values = data || ({} as any);
    const type = values.type || 'error';
    const title = values.title || 'Error';
    const message = values.message || 'Something went wrong. Please try again.';
    openNotification({ ...values, message, title, type });
  });

  const openInfoNotification = useMemoizedFn(
    ({ type = 'info', message = 'Info', title = 'Info', ...props }: NotificationProps) => {
      openNotification({ ...props, title, message, type });
    }
  );

  const openSuccessNotification = useMemoizedFn(
    ({ type = 'success', title = 'Success', message = 'success', ...props }: NotificationProps) => {
      openNotification({ ...props, message, title, type });
    }
  );

  const openWarningNotification = useMemoizedFn(
    ({ type = 'warning', title = 'Warning', message = 'Warning', ...props }: NotificationProps) => {
      openNotification({ ...props, message, title, type });
    }
  );

  //MESSAGES

  const openMessage = useMemoizedFn(
    (props: {
      type: NotificationType;
      message: string;
      loading?: boolean;
      onClose?: () => void;
      duration?: number;
    }) => {
      if (props.loading) {
        message.loading(props.message, props.duration, props.onClose);
      } else {
        message?.[props.type]?.(props.message, props.duration, props.onClose);
      }
    }
  );

  const openErrorMessage = useMemoizedFn((message: string) => {
    openMessage({ type: 'error', message });
  });

  const openInfoMessage = useMemoizedFn((message: string, duration?: number) => {
    openMessage({ type: 'info', message, duration });
  });

  const openSuccessMessage = useMemoizedFn((message: string) => {
    openMessage({ type: 'success', message });
  });

  const openConfirmModal = useMemoizedFn(
    (props: {
      title: string | React.ReactNode;
      content: string | React.ReactNode;
      onOk: () => void;
      onCancel?: () => void;
      icon?: React.ReactNode;
      okButtonProps?: ModalFuncProps['okButtonProps'];
      cancelButtonProps?: ModalFuncProps['cancelButtonProps'];
      width?: string | number;
      useReject?: boolean;
    }): Promise<void> => {
      const useReject = props.useReject ?? true;

      return new Promise((resolve, reject) => {
        modal.confirm({
          icon: props.icon || <></>,
          ...props,
          className: cx(styles.modal, ''),
          cancelButtonProps: {
            ...props.cancelButtonProps,
            type: 'text'
          },
          okButtonProps: {
            ...props.okButtonProps,
            type: 'default'
          },
          onOk: async () => {
            await props.onOk();
            resolve();
          },
          onCancel: async () => {
            await props.onCancel?.();
            if (useReject) reject();
            else resolve();
          }
        });
      });
    }
  );

  return {
    openErrorNotification,
    openInfoNotification,
    openSuccessNotification,
    openWarningNotification,
    openErrorMessage,
    openInfoMessage,
    openSuccessMessage,
    openConfirmModal
  };
};

const BusterNotifications = createContext<ReturnType<typeof useBusterNotificationsInternal>>(
  {} as ReturnType<typeof useBusterNotificationsInternal>
);

export const BusterNotificationsProvider: React.FC<PropsWithChildren> = ({ children }) => {
  const value = useBusterNotificationsInternal();

  return <BusterNotifications.Provider value={value}>{children}</BusterNotifications.Provider>;
};

const useBusterNotificationsSelector = <T,>(
  selector: ContextSelector<ReturnType<typeof useBusterNotificationsInternal>, T>
) => {
  return useContextSelector(BusterNotifications, selector);
};

export const useBusterNotifications = () => {
  const openConfirmModal = useBusterNotificationsSelector((state) => state.openConfirmModal);
  const openErrorNotification = useBusterNotificationsSelector(
    (state) => state.openErrorNotification
  );
  const openInfoNotification = useBusterNotificationsSelector(
    (state) => state.openInfoNotification
  );
  const openSuccessNotification = useBusterNotificationsSelector(
    (state) => state.openSuccessNotification
  );
  const openWarningNotification = useBusterNotificationsSelector(
    (state) => state.openWarningNotification
  );
  const openErrorMessage = useBusterNotificationsSelector((state) => state.openErrorMessage);
  const openInfoMessage = useBusterNotificationsSelector((state) => state.openInfoMessage);
  const openSuccessMessage = useBusterNotificationsSelector((state) => state.openSuccessMessage);

  return {
    openConfirmModal,
    openErrorNotification,
    openInfoNotification,
    openSuccessNotification,
    openWarningNotification,
    openErrorMessage,
    openInfoMessage,
    openSuccessMessage
  };
};
