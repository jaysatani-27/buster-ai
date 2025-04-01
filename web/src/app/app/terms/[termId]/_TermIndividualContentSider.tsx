import React from 'react';

import { useTermsContextSelector, useTermsIndividual } from '@/context/Terms';
import { BusterUserAvatar } from '@/components';
import { formatDate } from '@/utils';
import { DatasetList } from './_TermDatasetSelect';
import { Text } from '@/components';

export const TermIndividualContentSider: React.FC<{ termId: string }> = ({ termId }) => {
  const updateTerm = useTermsContextSelector((x) => x.updateTerm);
  const { term: selectedTerm } = useTermsIndividual({ termId });

  const datasets = selectedTerm?.datasets || [];

  const onChangeDatasets = async (datasets: string[]) => {
    const add_to_dataset = datasets.filter(
      (item) => !selectedTerm?.datasets?.some((dataset) => dataset.id === item)
    );
    const remove_from_dataset = selectedTerm?.datasets
      ?.filter((dataset) => !datasets.includes(dataset.id))
      .map((item) => item.id);

    await updateTerm({
      id: termId,
      add_to_dataset: add_to_dataset.length ? add_to_dataset : undefined,
      remove_from_dataset: remove_from_dataset.length ? remove_from_dataset : undefined
    });
  };

  return (
    <div className="h-full space-y-5 p-4">
      <div className="flex flex-col space-y-2.5">
        <Text type="secondary" className="!text-sm">
          Datasets that reference this term
        </Text>

        <DatasetList selectedDatasets={datasets} termId={termId} onChange={onChangeDatasets} />
      </div>

      <div className="space-y-2.5">
        <Text type="secondary" className="!text-sm">
          Created by
        </Text>

        <div className="flex items-center space-x-1.5">
          <BusterUserAvatar size={24} name={selectedTerm?.created_by.name} />
          <Text>{selectedTerm?.created_by.name}</Text>
          <Text type="secondary">
            (
            {formatDate({
              date: selectedTerm?.created_at!,
              format: 'LL'
            })}
            )
          </Text>
        </div>
      </div>
    </div>
  );
};
