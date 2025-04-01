import { AppContentHeader } from '@/app/app/_components/AppContentHeader';
import { ThreadControllerHeader } from '@/app/app/_controllers/ThreadController';
import React from 'react';

export default function Layout({
  children,
  params: { threadId, collectionId }
}: Readonly<{
  children: React.ReactNode;
  params: { threadId: string; collectionId: string };
}>) {
  return (
    <>
      <AppContentHeader>
        <ThreadControllerHeader threadId={threadId} collectionId={collectionId} />
      </AppContentHeader>
      {children}
    </>
  );
}
