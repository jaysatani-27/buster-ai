'use client';

import { createBusterRoute, BusterRoutes } from '@/routes';
import { isValidEmail, timeout } from '@/utils';
import { Button, Input, Result } from 'antd';
import Link from 'next/link';
import React, { useContext, useState } from 'react';
import { useStyles } from '../login/_components/LoginForm';
import { Title, Text } from '@/components';
import { useMemoizedFn } from 'ahooks';
import { useBusterNotifications } from '@/context/BusterNotifications';

export const ResetEmailForm: React.FC<{
  queryEmail: string;
  resetPasswordEmailSend: (d: { email: string }) => Promise<{ error: string } | undefined>;
}> = ({ queryEmail, resetPasswordEmailSend }) => {
  const [loading, setLoading] = useState(false);
  const [email, setEmail] = useState(queryEmail);
  const { styles, cx } = useStyles();
  const [emailSent, setEmailSent] = useState(false);
  const { openErrorNotification } = useBusterNotifications();

  const disabled = !email || !isValidEmail(email);

  const handleResetPassword = useMemoizedFn(async () => {
    if (disabled) return;
    setLoading(true);
    const [res] = await Promise.all([resetPasswordEmailSend({ email }), timeout(450)]);
    if (res?.error) {
      openErrorNotification(res.error);
    } else {
      setEmailSent(true);
    }
    setLoading(false);
  });

  if (emailSent) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-4">
        <Result
          status="success"
          title="Email sent"
          subTitle="Please check your email for the reset password link"
        />
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col items-center justify-center gap-4">
      <Title className="mb-0" level={1}>
        Reset Password
      </Title>

      <div className="flex w-[330px] flex-col gap-2">
        <Input
          placeholder="Email"
          value={email}
          onChange={(v) => {
            setEmail(v.target.value);
          }}
          onPressEnter={handleResetPassword}
        />

        <Button
          block
          loading={loading}
          type="primary"
          disabled={disabled}
          onClick={handleResetPassword}>
          Send reset password email
        </Button>
      </div>

      <Link
        className={cx(
          'flex w-full cursor-pointer justify-center text-center font-normal',
          styles.link
        )}
        href={createBusterRoute({
          route: BusterRoutes.AUTH_LOGIN
        })}>
        <Text type="primary" size="xxs">
          Return to login
        </Text>
      </Link>
    </div>
  );
};
