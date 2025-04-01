import {
  GetDatasetGroupDatasetsResponse,
  GetPermissionGroupDatasetsResponse,
  GetPermissionGroupUsersResponse,
  useUpdateDatasetGroupDatasets,
  useUpdatePermissionGroupDatasets
} from '@/api/buster_rest';
import { PermissionAssignedCell } from '@/app/app/_components/PermissionComponents';
import {
  BusterInfiniteList,
  BusterListColumn,
  BusterListRowItem,
  EmptyStateList,
  InfiniteListContainer
} from '@/components/list';
import { BusterRoutes, createBusterRoute } from '@/routes';
import { useMemoizedFn } from 'ahooks';
import React, { useMemo, useState } from 'react';
import { DatasetGroupDatasetSelectedPopup } from './DatasetGroupDatasetSelectedPopup';

export const DatasetGroupDatasetsListContainer: React.FC<{
  filteredDatasets: GetDatasetGroupDatasetsResponse[];
  datasetGroupId: string;
}> = React.memo(({ filteredDatasets, datasetGroupId }) => {
  const [selectedRowKeys, setSelectedRowKeys] = useState<string[]>([]);
  const { mutateAsync: updateDatasetGroupDatasets } = useUpdateDatasetGroupDatasets();

  const onSelectAssigned = useMemoizedFn(async (params: { id: string; assigned: boolean }) => {
    await updateDatasetGroupDatasets({
      datasetGroupId,
      groups: [params]
    });
  });

  const columns: BusterListColumn[] = useMemo(
    () => [
      {
        title: 'Name',
        dataIndex: 'name'
      },
      {
        title: 'Assigned',
        dataIndex: 'assigned',
        width: 130 + 85,
        render: (assigned: boolean, permissionGroup: GetPermissionGroupUsersResponse) => {
          return (
            <div className="flex justify-end">
              <PermissionAssignedCell
                id={permissionGroup.id}
                assigned={assigned}
                text="assigned"
                onSelect={onSelectAssigned}
              />
            </div>
          );
        }
      }
    ],
    []
  );

  const { cannotQueryPermissionUsers, canQueryPermissionUsers } = useMemo(() => {
    const result: {
      cannotQueryPermissionUsers: BusterListRowItem[];
      canQueryPermissionUsers: BusterListRowItem[];
    } = filteredDatasets.reduce<{
      cannotQueryPermissionUsers: BusterListRowItem[];
      canQueryPermissionUsers: BusterListRowItem[];
    }>(
      (acc, dataset) => {
        const datasetItem: BusterListRowItem = {
          id: dataset.id,
          data: dataset,
          link: createBusterRoute({
            route: BusterRoutes.APP_DATASETS_ID,
            datasetId: dataset.id
          })
        };
        if (dataset.assigned) {
          acc.canQueryPermissionUsers.push(datasetItem);
        } else {
          acc.cannotQueryPermissionUsers.push(datasetItem);
        }
        return acc;
      },
      {
        cannotQueryPermissionUsers: [] as BusterListRowItem[],
        canQueryPermissionUsers: [] as BusterListRowItem[]
      }
    );
    return result;
  }, [filteredDatasets]);

  const rows = useMemo(
    () => [
      {
        id: 'header-assigned',
        data: {},
        hidden: canQueryPermissionUsers.length === 0,
        rowSection: {
          title: 'Assigned',
          secondaryTitle: canQueryPermissionUsers.length.toString()
        }
      },
      ...canQueryPermissionUsers,
      {
        id: 'header-not-assigned',
        data: {},
        hidden: cannotQueryPermissionUsers.length === 0,
        rowSection: {
          title: 'Not Assigned',
          secondaryTitle: cannotQueryPermissionUsers.length.toString()
        }
      },
      ...cannotQueryPermissionUsers
    ],
    [canQueryPermissionUsers, cannotQueryPermissionUsers]
  );

  return (
    <InfiniteListContainer
      popupNode={
        <DatasetGroupDatasetSelectedPopup
          selectedRowKeys={selectedRowKeys}
          onSelectChange={setSelectedRowKeys}
          datasetGroupId={datasetGroupId}
        />
      }>
      <BusterInfiniteList
        columns={columns}
        rows={rows}
        showHeader={false}
        showSelectAll={false}
        useRowClickSelectChange={false}
        selectedRowKeys={selectedRowKeys}
        onSelectChange={setSelectedRowKeys}
        emptyState={<EmptyStateList text="No dataset groups found" />}
      />
    </InfiniteListContainer>
  );
});

DatasetGroupDatasetsListContainer.displayName = 'DatasetGroupDatasetsListContainer';
