'use client';

import { AppSegmented } from '@/components';
import { SegmentedProps } from 'antd';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import React, { useMemo } from 'react';
import { DatasetApps, DataSetAppText } from '../_config';
import { createBusterRoute, BusterRoutes } from '@/routes';
import { useMemoizedFn } from 'ahooks';
import { SegmentedValue } from 'antd/es/segmented';

export const DatasetsHeaderOptions: React.FC<{
  selectedApp: DatasetApps;
  isAdmin: boolean;
  datasetId: string | undefined;
}> = React.memo(({ datasetId, isAdmin, selectedApp }) => {
  const { push } = useRouter();
  const optionsItems = useMemo(
    () =>
      isAdmin
        ? [DatasetApps.OVERVIEW, DatasetApps.PERMISSIONS, DatasetApps.EDITOR]
        : [DatasetApps.OVERVIEW, DatasetApps.PERMISSIONS],
    [isAdmin]
  );

  const options = useMemo(
    () =>
      optionsItems.map((key) => ({
        label: DataSetAppText[key as DatasetApps],
        link: keyToRoute(datasetId!, key),
        value: key
      })),
    [datasetId, optionsItems]
  );

  const onChangeSegment = useMemoizedFn((value: SegmentedValue) => {
    if (datasetId) push(keyToRoute(datasetId, value as DatasetApps));
  });

  return <AppSegmented options={options} value={selectedApp} onChange={onChangeSegment} />;
});
DatasetsHeaderOptions.displayName = 'DatasetsHeaderOptions';

const keyToRoute = (datasetId: string, key: DatasetApps) => {
  const record: Record<DatasetApps, string> = {
    [DatasetApps.PERMISSIONS]: createBusterRoute({
      route: BusterRoutes.APP_DATASETS_ID_PERMISSIONS_OVERVIEW,
      datasetId
    }),
    [DatasetApps.OVERVIEW]: createBusterRoute({
      route: BusterRoutes.APP_DATASETS_ID_OVERVIEW,
      datasetId
    }),
    [DatasetApps.EDITOR]: createBusterRoute({
      route: BusterRoutes.APP_DATASETS_ID_EDITOR,
      datasetId
    })
  };

  return record[key];
};
