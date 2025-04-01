'use client';

import { AppSegmented } from '@/components/segmented';
import { useAppLayoutContextSelector } from '@/context/BusterAppLayout';
import { createBusterRoute, BusterRoutes } from '@/routes/busterRoutes';
import { useDebounce } from 'ahooks';
import { Divider } from 'antd';
import React from 'react';

export enum PermissionSegmentsApps {
  USERS = 'users',
  DATASET_GROUPS = 'dataset-groups',
  DATASETS = 'datasets'
}

const RouteToAppSegment: Record<string, PermissionSegmentsApps> = {
  [BusterRoutes.SETTINGS_PERMISSION_GROUPS_ID_USERS]: PermissionSegmentsApps.USERS,
  [BusterRoutes.SETTINGS_PERMISSION_GROUPS_ID_DATASET_GROUPS]:
    PermissionSegmentsApps.DATASET_GROUPS,
  [BusterRoutes.SETTINGS_PERMISSION_GROUPS_ID_DATASETS]: PermissionSegmentsApps.DATASETS
};

export const PermissionAppSegments: React.FC<{
  permissionGroupId: string;
}> = ({ permissionGroupId }) => {
  const route = useAppLayoutContextSelector((state) => state.currentRoute);
  const debouncedRoute = useDebounce(route, { wait: 10 });
  const value = RouteToAppSegment[debouncedRoute] || PermissionSegmentsApps.USERS;

  const options = React.useMemo(
    () => [
      {
        label: 'Users',
        value: PermissionSegmentsApps.USERS,
        link: createBusterRoute({
          route: BusterRoutes.SETTINGS_PERMISSION_GROUPS_ID_USERS,
          permissionGroupId
        })
      },
      {
        label: 'Dataset groups',
        value: PermissionSegmentsApps.DATASET_GROUPS,
        link: createBusterRoute({
          route: BusterRoutes.SETTINGS_PERMISSION_GROUPS_ID_DATASET_GROUPS,
          permissionGroupId
        })
      },
      {
        label: 'Datasets',
        value: PermissionSegmentsApps.DATASETS,
        link: createBusterRoute({
          route: BusterRoutes.SETTINGS_PERMISSION_GROUPS_ID_DATASETS,
          permissionGroupId
        })
      }
    ],
    [permissionGroupId]
  );

  return (
    <div className="flex flex-col space-y-2">
      <AppSegmented value={value} options={options} />
      <Divider />
    </div>
  );
};
