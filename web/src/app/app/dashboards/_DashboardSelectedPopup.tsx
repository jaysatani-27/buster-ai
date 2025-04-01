import React, { useState } from 'react';
import { AppMaterialIcons } from '@/components';
import { BusterListSelectedOptionPopupContainer } from '@/components/list';
import { Button, Dropdown, DropdownProps } from 'antd';
import { useUserConfigContextSelector } from '@/context/Users';
import { useCollectionsContextSelector } from '@/context/Collections';
import { useMemoizedFn, useMount } from 'ahooks';
import { useBusterNotifications } from '@/context/BusterNotifications';
import { SaveToCollectionsDropdown } from '../_components/Buttons';
import { useDashboardContextSelector } from '@/context/Dashboards';

export const DashboardSelectedOptionPopup: React.FC<{
  selectedRowKeys: string[];
  onSelectChange: (selectedRowKeys: string[]) => void;
  hasSelected: boolean;
}> = ({ selectedRowKeys, onSelectChange, hasSelected }) => {
  return (
    <BusterListSelectedOptionPopupContainer
      selectedRowKeys={selectedRowKeys}
      onSelectChange={onSelectChange}
      buttons={[
        <CollectionsButton
          key="collections"
          selectedRowKeys={selectedRowKeys}
          onSelectChange={onSelectChange}
        />,
        <DeleteButton
          key="delete"
          selectedRowKeys={selectedRowKeys}
          onSelectChange={onSelectChange}
        />,
        <ThreeDotButton
          key="three-dot"
          selectedRowKeys={selectedRowKeys}
          onSelectChange={onSelectChange}
        />
      ]}
      show={hasSelected}
    />
  );
};

const CollectionsButton: React.FC<{
  selectedRowKeys: string[];
  onSelectChange: (selectedRowKeys: string[]) => void;
}> = ({ selectedRowKeys, onSelectChange }) => {
  const { openInfoMessage } = useBusterNotifications();
  const onAddToCollection = useDashboardContextSelector((state) => state.onAddToCollection);
  const onRemoveFromCollection = useDashboardContextSelector(
    (state) => state.onRemoveFromCollection
  );

  const collectionsList = useCollectionsContextSelector((state) => state.collectionsList);
  const getInitialCollections = useCollectionsContextSelector(
    (state) => state.getInitialCollections
  );

  const [selectedCollections, setSelectedCollections] = useState<
    Parameters<typeof SaveToCollectionsDropdown>[0]['selectedCollections']
  >([]);

  const onSaveToCollection = useMemoizedFn(async (collectionIds: string[]) => {
    setSelectedCollections(collectionIds);
    const allSaves: Promise<void>[] = selectedRowKeys.map((dashboardId) => {
      return onAddToCollection({
        dashboardId,
        collectionId: collectionIds
      });
    });
    await Promise.all(allSaves);
    openInfoMessage('Dashboards saved to collections');
  });

  const onRemoveFromCollectionPreflight = useMemoizedFn(async (collectionId: string) => {
    setSelectedCollections((prev) => prev.filter((id) => id !== collectionId));
    const allRemoves = selectedRowKeys.map((dashboardId) => {
      return onRemoveFromCollection({ dashboardId, collectionId });
    });

    await Promise.all(allRemoves);
    openInfoMessage('Dashboards removed from collections');
  });

  useMount(() => {
    if (!collectionsList.length) getInitialCollections();
  });

  return (
    <SaveToCollectionsDropdown
      onSaveToCollection={onSaveToCollection}
      onRemoveFromCollection={onRemoveFromCollectionPreflight}
      selectedCollections={selectedCollections}>
      <Button icon={<AppMaterialIcons icon="note_stack" />} type="default">
        Collections
      </Button>
    </SaveToCollectionsDropdown>
  );
};

const DeleteButton: React.FC<{
  selectedRowKeys: string[];
  onSelectChange: (selectedRowKeys: string[]) => void;
}> = ({ selectedRowKeys, onSelectChange }) => {
  const onDeleteDashboard = useDashboardContextSelector((state) => state.onDeleteDashboard);
  const { openConfirmModal } = useBusterNotifications();

  const onDeleteClick = useMemoizedFn(async () => {
    openConfirmModal({
      title: 'Delete thread',
      content: 'Are you sure you want to delete these threads?',
      onOk: async () => {
        await onDeleteDashboard(selectedRowKeys, true);
        onSelectChange([]);
      }
    });
  });

  return (
    <Button icon={<AppMaterialIcons icon="delete" />} type="default" onClick={onDeleteClick}>
      Delete
    </Button>
  );
};

const ThreeDotButton: React.FC<{
  selectedRowKeys: string[];
  onSelectChange: (selectedRowKeys: string[]) => void;
}> = ({ selectedRowKeys, onSelectChange }) => {
  const bulkEditFavorites = useUserConfigContextSelector((state) => state.bulkEditFavorites);
  const userFavorites = useUserConfigContextSelector((state) => state.userFavorites);

  const dropdownOptions: Required<DropdownProps>['menu']['items'] = [
    {
      label: 'Add to favorites',
      icon: <AppMaterialIcons icon="star" />,
      key: 'add-to-favorites',
      onClick: async () => {
        const allFavorites: string[] = [...userFavorites.map((f) => f.id), ...selectedRowKeys];
        //    bulkEditFavorites(allFavorites);
        alert('TODO - feature not implemented yet');
      }
    },
    {
      label: 'Remove from favorites',
      icon: <AppMaterialIcons icon="close" />,
      key: 'remove-from-favorites',
      onClick: async () => {
        const allFavorites: string[] = userFavorites
          .map((f) => f.id)
          .filter((id) => !selectedRowKeys.includes(id));
        bulkEditFavorites(allFavorites);
      }
    }
  ];

  return (
    <Dropdown menu={{ items: dropdownOptions }} trigger={['click']}>
      <Button icon={<AppMaterialIcons icon="more_horiz" />} type="default" />
    </Dropdown>
  );
};
