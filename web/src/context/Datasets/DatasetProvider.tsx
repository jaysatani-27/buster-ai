import React, { PropsWithChildren, useState } from 'react';
import { useParams } from 'next/navigation';
import {
  ContextSelector,
  createContext,
  useContextSelector
} from '@fluentui/react-context-selector';

export const useDatasets = () => {
  const { openedDatasetId } = useParams<{ openedDatasetId: string }>();
  const [openNewDatasetModal, setOpenNewDatasetModal] = useState(false);

  return {
    openedDatasetId,
    openNewDatasetModal,
    setOpenNewDatasetModal
  };
};

const BusterDatasets = createContext<ReturnType<typeof useDatasets>>(
  {} as ReturnType<typeof useDatasets>
);

export const DatasetProviders: React.FC<PropsWithChildren> = ({ children }) => {
  const Datasets = useDatasets();

  return <BusterDatasets.Provider value={Datasets}>{children}</BusterDatasets.Provider>;
};

export const useDatasetContextSelector = <T,>(
  selector: ContextSelector<ReturnType<typeof useDatasets>, T>
) => useContextSelector(BusterDatasets, selector);
