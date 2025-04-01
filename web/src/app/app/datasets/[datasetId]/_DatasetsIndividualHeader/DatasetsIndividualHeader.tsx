'use client';

import React, { useMemo } from 'react';
import { Button, Divider } from 'antd';
import { PreventNavigation } from '@/components';
import { useDatasetContextSelector } from '@/context/Datasets';
import { useHotkeys } from 'react-hotkeys-hook';
import { AppContentHeader } from '../../../_components/AppContentHeader';
import { useMemoizedFn } from 'ahooks';
import { useUserConfigContextSelector } from '@/context/Users';
import { DatasetsHeaderOptions } from './DatasetHeaderOptions';
import { DatasetBreadcrumb } from './DatasetBreadcrumb';
import { DatasetIndividualThreeDotMenu } from './DatasetIndividualThreeDotMenu';
import { useDatasetPageContextSelector } from '../_DatasetPageContext';

export const DatasetsIndividualHeader: React.FC<{}> = React.memo(({}) => {
  const selectedApp = useDatasetPageContextSelector((state) => state.selectedApp);
  const dataset = useDatasetPageContextSelector((state) => state.dataset);
  const onPublishDataset = useDatasetPageContextSelector((state) => state.onPublishDataset);
  const isDeployingDataset = useDatasetPageContextSelector((state) => state.isDeployingDataset);
  const resetDataset = useDatasetPageContextSelector((state) => state.resetDataset);
  const disablePublish = useDatasetPageContextSelector((state) => state.disablePublish);
  const isChangedSQL = useDatasetPageContextSelector((state) => state.isChangedSQL);
  const datasetId = useDatasetPageContextSelector((state) => state.datasetId);

  const datasetName = dataset?.data?.name;

  const isAdmin = useUserConfigContextSelector((state) => state.isAdmin);

  const preventNavigation = !disablePublish;

  const onReset = useMemoizedFn(() => {
    resetDataset();
  });

  const onCancelPreventNavigation = useMemoizedFn(async () => {
    setTimeout(() => {
      onReset();
    }, 300);
  });

  return (
    <>
      <AppContentHeader className="items-center justify-between space-x-2">
        <div className="flex items-center space-x-3">
          <DatasetBreadcrumb datasetName={datasetName} />

          <DatasetsHeaderOptions
            isAdmin={isAdmin}
            selectedApp={selectedApp}
            datasetId={datasetId}
          />
        </div>

        <div className="flex items-center">
          <div className="flex items-center">
            <DatasetIndividualThreeDotMenu datasetId={datasetId} />

            <Divider type="vertical" className="!h-4" />

            <div className="flex items-center space-x-2">
              <Button type="text" onClick={onReset} disabled={!isChangedSQL || isDeployingDataset}>
                Reset
              </Button>
              <Button
                type="primary"
                disabled={disablePublish}
                onClick={onPublishDataset}
                loading={isDeployingDataset}>
                Publish
              </Button>
            </div>
          </div>
        </div>
      </AppContentHeader>

      <PreventNavigation
        isDirty={preventNavigation}
        title="Would you like to publish your changes to this dataset?"
        description="You are about to leave this page without publishing changes. Would you like to publish your changes before you leave?"
        okText="Publish changes"
        cancelText="Discard changes"
        onOk={onPublishDataset}
        onCancel={onCancelPreventNavigation}
        doNotLeavePageOnOkay
      />
    </>
  );
});

DatasetsIndividualHeader.displayName = 'DatasetsIndividualHeader';
