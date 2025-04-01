'use client';

import React, { useContext } from 'react';
import { DashboardIndividualHeaderPrimary } from './DashboardInvidualHeaderPrimary';
import { AppContentHeader } from '../../_components/AppContentHeader';
import { useDashboardContextSelector, useIndividualDashboard } from '@/context/Dashboards';
import { Text } from '@/components';
import { Button } from 'antd';
import Link from 'next/link';
import { BusterRoutes, createBusterRoute } from '@/routes';
import { BusterLogoNew } from '@/assets/svg/BusterLogoNew';
import { useUserConfigContextSelector } from '@/context/Users';

export const DashboardIndividualHeader: React.FC<{}> = () => {
  const isAnonymousUser = useUserConfigContextSelector((state) => state.isAnonymousUser);

  if (isAnonymousUser) {
    return <DashboardAnonmousHeader />;
  }

  return <DashboardIndividualHeaderPrimary />;
};

const DashboardAnonmousHeader: React.FC<{}> = () => {
  const openedDashboardId = useDashboardContextSelector((x) => x.openedDashboardId);
  const { dashboardResponse } = useIndividualDashboard({ dashboardId: openedDashboardId });

  return (
    <AppContentHeader>
      <div className="flex w-full items-center justify-between">
        <Link href={createBusterRoute({ route: BusterRoutes.APP_DASHBOARDS })}>
          <BusterLogoNew
            style={{
              height: 24
            }}
          />
        </Link>
        <Text>{dashboardResponse?.dashboard?.name}</Text>
        <Link
          href={createBusterRoute({
            route: BusterRoutes.AUTH_LOGIN
          })}>
          <Button>Sign in</Button>
        </Link>
      </div>
    </AppContentHeader>
  );
};
