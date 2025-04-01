import { AppContentHeader } from '@/app/app/_components/AppContentHeader';
import { ThreadControllerHeader } from '@/app/app/_controllers/ThreadController';
import React from 'react';

export default function Layout({
  children,
  params: { threadId, dashboardId }
}: Readonly<{
  children: React.ReactNode;
  params: { threadId: string; dashboardId: string };
}>) {
  return (
    <>
      <AppContentHeader>
        <ThreadControllerHeader threadId={threadId} dashboardId={dashboardId} />
      </AppContentHeader>
      {children}
    </>
  );
}
