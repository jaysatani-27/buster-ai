import React from 'react';
import Link from 'next/link';
import { Text } from '@/components/text';
import { BusterRoutes, createBusterRoute } from '@/routes';

export const ThreadControllerDatasetLink: React.FC<{
  datasetName: string;
  datasetId: string;
}> = React.memo(({ datasetName, datasetId }) => {
  return (
    <Link
      className=""
      href={createBusterRoute({
        route: BusterRoutes.APP_DATASETS_ID_OVERVIEW,
        datasetId: datasetId
      })}>
      <Text type="tertiary" className="!hover:text-black dark:!hover:text-white">
        Dataset: {datasetName}
      </Text>
    </Link>
  );
});
ThreadControllerDatasetLink.displayName = 'ThreadControllerDatasetLink';
