import React from 'react';
import { Text } from '@/components';
import { Button, Skeleton } from 'antd';
import Link from 'next/link';
import { BusterRoutes, createBusterRoute } from '@/routes';
import { useBusterThreadIndividual, useBusterThreadMessage } from '@/context/Threads';

export const ThreadControllerHeaderAnon: React.FC<{ threadId: string }> = ({ threadId }) => {
  const thread = useBusterThreadIndividual({ threadId });
  const { message: currentMessage } = useBusterThreadMessage({ threadId });

  if (!thread || !currentMessage) {
    return <SkeletonLoader />;
  }

  return (
    <div className="flex w-full items-center justify-between">
      <Text>{currentMessage?.title}</Text>

      <div className="flex items-center space-x-2">
        <Link
          prefetch={true}
          href={createBusterRoute({
            route: BusterRoutes.AUTH_LOGIN
          })}>
          <Button>Sign in</Button>
        </Link>
      </div>
    </div>
  );
};

const SkeletonLoader: React.FC = () => {
  return (
    <div className="flex w-full items-center justify-between space-x-2">
      <Skeleton.Input
        block
        size="small"
        className="!flex h-full w-24 max-w-[45vw] !items-center overflow-hidden rounded"
      />

      <Skeleton.Input
        size="small"
        className="!flex h-full w-12 !items-center overflow-hidden rounded"
      />
    </div>
  );
};
