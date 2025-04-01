import { AppMaterialIcons } from '@/components';
import { useMemoizedFn } from 'ahooks';
import { MenuProps, Dropdown, Button } from 'antd';
import React, { useMemo } from 'react';
import { PERMISSION_OPTIONS_INCLUDED, PERMISSION_OPTIONS_ASSIGNED } from './PermissionAssignedCell';

export const PermissionAssignedButton: React.FC<{
  selectedRowKeys: string[];
  onSelectChange: (selectedRowKeys: string[]) => void;
  text: 'assigned' | 'included';
  onUpdate: (groups: { id: string; assigned: boolean }[]) => Promise<void>;
}> = ({ selectedRowKeys, text, onUpdate, onSelectChange }) => {
  const options = useMemo(() => {
    const selectedOptions =
      text === 'included' ? PERMISSION_OPTIONS_INCLUDED : PERMISSION_OPTIONS_ASSIGNED;
    return selectedOptions.map((v) => ({
      ...v,
      icon: v.value ? <AppMaterialIcons icon="done_all" /> : <AppMaterialIcons icon="remove_done" />
    }));
  }, [text]);

  const buttonText = useMemo(() => {
    return text === 'included' ? 'Include' : 'Assign';
  }, [text]);

  const onAssignClick = useMemoizedFn(async (assigned: boolean) => {
    try {
      const groups: { id: string; assigned: boolean }[] = selectedRowKeys.map((v) => ({
        id: v,
        assigned
      }));
      await onUpdate(groups);
      onSelectChange([]);
    } catch (error) {
      //  openErrorMessage('Failed to delete collection');
    }
  });

  const menuProps: MenuProps = useMemo(() => {
    return {
      selectable: true,
      items: options.map((v) => ({
        icon: v.icon,
        label: v.label,
        key: v.value ? 'included' : 'not_included',
        onClick: () => onAssignClick(v.value)
      }))
    };
  }, [selectedRowKeys]);

  const onButtonClick = useMemoizedFn((e: React.MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation();
    e.preventDefault();
  });

  return (
    <Dropdown menu={menuProps} trigger={['click']}>
      <Button type="default" onClick={onButtonClick}>
        {buttonText}
      </Button>
    </Dropdown>
  );
};
