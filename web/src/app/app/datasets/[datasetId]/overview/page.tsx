'use client';

import React from 'react';
import { useDatasetPageContextSelector } from '../_DatasetPageContext';
import { OverviewHeader } from './OverviewHeader';
import { OverviewData } from './OverviewData';
import { Divider } from 'antd';

export default function Page() {
  const datasetRes = useDatasetPageContextSelector((state) => state.dataset);
  const datasetDataRes = useDatasetPageContextSelector((state) => state.datasetData);

  const datasetData = datasetDataRes?.data;
  const dataset = datasetRes?.data;
  const isFetchedDataset = datasetRes?.isFetched;
  const isFetchedDatasetData = datasetDataRes?.isFetched;

  const showSkeletonLoader = !dataset?.id || !isFetchedDataset;

  return (
    <div className="mx-auto overflow-y-auto px-14 pb-12 pt-12">
      <>
        {showSkeletonLoader ? (
          <></>
        ) : (
          <div className="flex w-full flex-col space-y-5">
            <OverviewHeader
              datasetId={dataset.id}
              description={dataset.description}
              name={dataset.name}
            />

            <Divider />

            <OverviewData
              datasetId={dataset.id}
              data={datasetData || []}
              isFetchedDatasetData={isFetchedDatasetData}
            />
          </div>
        )}
      </>
    </div>
  );
}
