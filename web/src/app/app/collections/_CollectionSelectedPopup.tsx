import { AppMaterialIcons } from '@/components';
import { BusterListSelectedOptionPopupContainer } from '@/components/list';
import { useCollectionsContextSelector } from '@/context/Collections';
import { useMemoizedFn } from 'ahooks';
import { Button } from 'antd';
import React from 'react';

export const CollectionSelectedPopup: React.FC<{
  selectedRowKeys: string[];
  onSelectChange: (selectedRowKeys: string[]) => void;
}> = ({ selectedRowKeys, onSelectChange }) => {
  const show = selectedRowKeys.length > 0;

  return (
    <BusterListSelectedOptionPopupContainer
      selectedRowKeys={selectedRowKeys}
      onSelectChange={onSelectChange}
      buttons={[
        <CollectionDeleteButton
          key="delete"
          selectedRowKeys={selectedRowKeys}
          onSelectChange={onSelectChange}
        />
      ]}
      show={show}
    />
  );
};

const CollectionDeleteButton: React.FC<{
  selectedRowKeys: string[];
  onSelectChange: (selectedRowKeys: string[]) => void;
}> = ({ selectedRowKeys, onSelectChange }) => {
  const deleteCollection = useCollectionsContextSelector((x) => x.deleteCollection);

  const onDeleteClick = useMemoizedFn(async () => {
    try {
      const deletePromises = selectedRowKeys.map((v) => deleteCollection(v, false));
      await Promise.all(deletePromises);
      onSelectChange([]);
    } catch (error) {
      //  openErrorMessage('Failed to delete collection');
    }
  });

  return (
    <Button icon={<AppMaterialIcons icon="delete" />} type="default" onClick={onDeleteClick}>
      Delete
    </Button>
  );
};
