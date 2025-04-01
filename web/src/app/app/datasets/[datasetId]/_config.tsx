import { AppMaterialIcons } from '@/components';
import React from 'react';

export enum DatasetApps {
  OVERVIEW = 'overview',
  PERMISSIONS = 'permissions',
  EDITOR = 'editor'
}

export const DataSetAppText: Record<DatasetApps, string> = {
  [DatasetApps.OVERVIEW]: 'Overview',
  [DatasetApps.PERMISSIONS]: 'Permissions',
  [DatasetApps.EDITOR]: 'Editor'
};

export const DataSetAppIcons: Record<DatasetApps, React.ReactNode> = {
  [DatasetApps.OVERVIEW]: <AppMaterialIcons icon="info" />,
  [DatasetApps.PERMISSIONS]: <AppMaterialIcons icon="menu_book" />,
  [DatasetApps.EDITOR]: <AppMaterialIcons icon="data_object" />
};
