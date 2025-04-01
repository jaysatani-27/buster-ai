'use client';

import React, { useContext } from 'react';
import { ThreadControllerHeaderPrimary } from './ThreadControllerHeaderPrimary';
import { ThreadControllerHeaderAnon } from './ThreadControllerHeaderAnon';
import { useUserConfigContextSelector } from '@/context/Users';

export const ThreadControllerHeader: React.FC<{
  threadId: string;
  dashboardId?: string;
  collectionId?: string;
}> = ({ dashboardId, threadId, collectionId }) => {
  const isAnonymousUser = useUserConfigContextSelector((state) => state.isAnonymousUser);

  if (isAnonymousUser) {
    return <ThreadControllerHeaderAnon threadId={threadId} />;
  }

  return (
    <ThreadControllerHeaderPrimary
      dashboardId={dashboardId}
      threadId={threadId}
      collectionId={collectionId}
    />
  );
};
