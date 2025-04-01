import React from 'react';
import { BusterListSelectedOptionPopupContainer } from '@/components/list';
import { Button } from 'antd';
import { useMemoizedFn } from 'ahooks';
import { useTermsContextSelector } from '@/context/Terms';
import { useBusterNotifications } from '@/context/BusterNotifications';

export const TermListSelectedOptionPopup: React.FC<{
  selectedRowKeys: string[];
  onSelectChange: (selectedRowKeys: string[]) => void;
}> = ({ selectedRowKeys, onSelectChange }) => {
  return (
    <BusterListSelectedOptionPopupContainer
      selectedRowKeys={selectedRowKeys}
      onSelectChange={onSelectChange}
      buttons={[
        <DeleteButton
          key="delete"
          selectedRowKeys={selectedRowKeys}
          onSelectChange={onSelectChange}
        />
      ]}
      show={selectedRowKeys.length > 0}
    />
  );
};

const DeleteButton: React.FC<{
  selectedRowKeys: string[];
  onSelectChange: (selectedRowKeys: string[]) => void;
}> = ({ selectedRowKeys, onSelectChange }) => {
  const deleteTerm = useTermsContextSelector((x) => x.deleteTerm);
  const { openConfirmModal } = useBusterNotifications();

  const onDeleteClick = useMemoizedFn(async () => {
    return openConfirmModal({
      title: 'Delete terms',
      content: 'Are you sure you want to delete these terms?',
      onOk: async () => {
        await Promise.all(selectedRowKeys.map((id) => deleteTerm({ id }, true)));
        onSelectChange([]);
      }
    });
  });

  return (
    <Button type="default" onClick={onDeleteClick}>
      Delete
    </Button>
  );
};
