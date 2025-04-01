'use client';

import React, { useContext, useState } from 'react';
import { Button, Input } from 'antd';
import { Title, Text } from '@/components/text';
import { AnimatePresence, motion } from 'framer-motion';
import { useUserConfigContextSelector } from '@/context/Users';
import { inputHasText } from '@/utils';
import { useMemoizedFn } from 'ahooks';
import { useAppLayoutContextSelector } from '@/context/BusterAppLayout';
import { BusterAppRoutes } from '@/routes/busterRoutes/busterAppRoutes';
import { useBusterNotifications } from '@/context/BusterNotifications';

export const NewUserController = () => {
  const [started, setStarted] = useState(false);
  const onChangePage = useAppLayoutContextSelector((s) => s.onChangePage);
  const onCreateUserOrganization = useUserConfigContextSelector((s) => s.onCreateUserOrganization);
  const user = useUserConfigContextSelector((s) => s.user);
  const userOrganizations = useUserConfigContextSelector((s) => s.userOrganizations);
  const { openInfoMessage } = useBusterNotifications();
  const [submitting, setSubmitting] = useState(false);
  const [name, setName] = useState<string | undefined>(user?.name);
  const [company, setCompany] = useState<string | undefined>(userOrganizations?.name);

  const canSubmit = inputHasText(name) && inputHasText(company);

  const handleSubmit = useMemoizedFn(async () => {
    if (!canSubmit || !name || !company) {
      openInfoMessage('Please fill in all fields');
      return;
    }
    setSubmitting(true);
    try {
      await onCreateUserOrganization({
        name,
        company,
        alreadyHasCompany: !!userOrganizations?.name
      });

      onChangePage({
        route: BusterAppRoutes.APP_THREAD
      });
    } catch (error) {
      //
    }

    setTimeout(() => {
      setSubmitting(false);
    }, 350);
  });

  return (
    <AnimatePresence mode="wait" initial={false}>
      {!started && (
        <motion.div
          initial={{ opacity: 0, y: -30 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: 30, transition: { duration: 0.2, opacity: { duration: 0.175 } } }}
          key={'no-started'}
          className="flex h-full w-full flex-col items-start justify-center space-y-5 p-12">
          <Title level={3}>Welcome to Buster</Title>
          <Text type="secondary">
            With Buster, you can ask data questions in plain english & instantly get back data.
          </Text>
          <Button type="primary" onClick={() => setStarted(true)}>
            Get Started
          </Button>
        </motion.div>
      )}

      {started && (
        <motion.div
          initial={{ opacity: 0, y: -30 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: 0 }}
          transition={{ duration: 0.2 }}
          key={'started'}
          className="flex h-full w-full flex-col items-start justify-center space-y-5 p-12">
          <Title level={4}>Tell us about yourself</Title>
          <Input
            placeholder="What is your full name"
            className="w-full"
            value={name}
            defaultValue={user?.name}
            name="name"
            onChange={(e) => setName(e.target.value)}
          />
          <Input
            placeholder="What is the name of your company"
            className="w-full"
            name="company"
            disabled={!!userOrganizations?.name}
            value={company}
            onChange={(e) => setCompany(e.target.value)}
            onPressEnter={handleSubmit}
            defaultValue={userOrganizations?.name}
          />
          <Button
            type="primary"
            loading={submitting}
            onClick={async () => {
              handleSubmit();
            }}>
            Create your account
          </Button>
        </motion.div>
      )}
    </AnimatePresence>
  );
};
