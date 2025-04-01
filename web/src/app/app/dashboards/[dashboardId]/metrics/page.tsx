import { BusterRoutes, createBusterRoute } from '@/routes';
import { permanentRedirect, redirect } from 'next/navigation';
import React from 'react';

export default function DashboardPage() {
  return permanentRedirect(
    createBusterRoute({
      route: BusterRoutes.APP_DASHBOARDS
    })
  );
}
