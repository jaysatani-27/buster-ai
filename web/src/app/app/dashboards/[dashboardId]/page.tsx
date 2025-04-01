import React from 'react';
import { DashboardIndividualHeader } from './_DashboardIndividualHeader';
import { DashboardIndividualContent } from './_DashboardIndividualContent';
import { AppAssetCheckLayout } from '../../_layouts';

export default function DashboardPage({
  params: { dashboardId }
}: {
  params: {
    dashboardId: string;
  };
}) {
  return (
    <AppAssetCheckLayout dashboardId={dashboardId} type="dashboard">
      <DashboardIndividualHeader />
      <DashboardIndividualContent />
    </AppAssetCheckLayout>
  );
}
