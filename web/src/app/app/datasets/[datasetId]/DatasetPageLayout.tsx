'use client';

import React from 'react';
import { DatasetsIndividualHeader } from './_DatasetsIndividualHeader';
import { AppContent } from '../../_components/AppContent';
import { DatasetPageProvider, useDatasetPageContextSelector } from './_DatasetPageContext';

export const DatasetPageLayout: React.FC<{ children: React.ReactNode; datasetId: string }> = ({
  children,
  datasetId
}) => {
  return (
    <DatasetPageProvider datasetId={datasetId}>
      <LayoutContent>{children}</LayoutContent>
    </DatasetPageProvider>
  );
};

const LayoutContent: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  return (
    <>
      <DatasetsIndividualHeader />
      <AppContent>{children}</AppContent>
    </>
  );
};
