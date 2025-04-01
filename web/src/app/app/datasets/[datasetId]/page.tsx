'use client';
import React from 'react';
import { permanentRedirect, RedirectType } from 'next/navigation';
import { BusterRoutes, createBusterRoute } from '@/routes';

export default function DashboardPage({
  params: { datasetId }
}: {
  params: {
    datasetId: string;
  };
}) {
  permanentRedirect(
    createBusterRoute({
      route: BusterRoutes.APP_DATASETS_ID_OVERVIEW,
      datasetId
    }),
    RedirectType.replace
  );

  return <></>;
}
