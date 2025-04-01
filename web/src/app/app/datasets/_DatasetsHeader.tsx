'use client';

import React, { useMemo } from 'react';
import { Breadcrumb, Button } from 'antd';
import Link from 'next/link';
import { BusterRoutes, createBusterRoute } from '@/routes';
import { AppMaterialIcons, AppSegmented, AppTooltip } from '@/components';
import { NewDatasetModal } from '@appComponents/NewDatasetModal';
import { AppContentHeader } from '../_components/AppContentHeader';
import { useDatasetContextSelector, useIndividualDataset } from '@/context/Datasets';
import { useHotkeys } from 'react-hotkeys-hook';
import { useUserConfigContextSelector } from '@/context/Users';
import { useMemoizedFn } from 'ahooks';

export const DatasetHeader: React.FC<{
  datasetFilter: 'all' | 'published' | 'drafts';
  setDatasetFilter: (filter: 'all' | 'published' | 'drafts') => void;
}> = React.memo(({ datasetFilter, setDatasetFilter }) => {
  const openedDatasetId = useDatasetContextSelector((state) => state.openedDatasetId);
  const openNewDatasetModal = useDatasetContextSelector((state) => state.openNewDatasetModal);
  const setOpenNewDatasetModal = useDatasetContextSelector((state) => state.setOpenNewDatasetModal);
  const isAdmin = useUserConfigContextSelector((state) => state.isAdmin);
  const { dataset } = useIndividualDataset({ datasetId: openedDatasetId });
  const datasetTitle = dataset?.data?.name || 'Datasets';

  const breadcrumbItems = useMemo(
    () =>
      [
        {
          title: (
            <Link
              suppressHydrationWarning
              href={
                openedDatasetId
                  ? createBusterRoute({
                      route: BusterRoutes.APP_DATASETS_ID_OVERVIEW,
                      datasetId: openedDatasetId
                    })
                  : createBusterRoute({ route: BusterRoutes.APP_DATASETS })
              }>
              {datasetTitle}
            </Link>
          )
        }
      ].filter((item) => item.title),
    [openedDatasetId, datasetTitle]
  );

  const onCloseNewDatasetModal = useMemoizedFn(() => {
    setOpenNewDatasetModal(false);
  });

  const onOpenNewDatasetModal = useMemoizedFn(() => {
    setOpenNewDatasetModal(true);
  });

  useHotkeys('d', onOpenNewDatasetModal);

  return (
    <>
      <AppContentHeader className="items-center justify-between space-x-2">
        <div className="flex space-x-3">
          <Breadcrumb className="flex items-center" items={breadcrumbItems} />
          <DatasetFilters datasetFilter={datasetFilter} setDatasetFilter={setDatasetFilter} />
        </div>

        <div className="flex items-center">
          {isAdmin && (
            <AppTooltip title={'Create new dashboard'} shortcuts={['D']}>
              <Button
                type="default"
                icon={<AppMaterialIcons icon="add" />}
                onClick={onOpenNewDatasetModal}>
                New Dataset
              </Button>
            </AppTooltip>
          )}
        </div>
      </AppContentHeader>

      {isAdmin && (
        <NewDatasetModal
          open={openNewDatasetModal}
          onClose={onCloseNewDatasetModal}
          datasourceId={openedDatasetId}
        />
      )}
    </>
  );
});
DatasetHeader.displayName = 'DatasetHeader';

const DatasetFilters: React.FC<{
  datasetFilter: 'all' | 'published' | 'drafts';
  setDatasetFilter: (filter: 'all' | 'published' | 'drafts') => void;
}> = ({ datasetFilter, setDatasetFilter }) => {
  const options: { label: string; value: 'all' | 'published' | 'drafts' }[] = useMemo(
    () => [
      { label: 'All', value: 'all' },
      { label: 'Published', value: 'published' },
      { label: 'Drafts', value: 'drafts' }
    ],
    []
  );

  return (
    <AppSegmented
      options={options}
      value={datasetFilter}
      onChange={(value) => {
        setDatasetFilter(value as 'all' | 'published' | 'drafts');
      }}
    />
  );
};
