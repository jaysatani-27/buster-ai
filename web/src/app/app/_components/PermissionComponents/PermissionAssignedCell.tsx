import { Select } from 'antd';
import React from 'react';

export const PERMISSION_OPTIONS_INCLUDED = [
  {
    label: 'Included',
    value: true
  },
  {
    label: 'Not Included',
    value: false
  }
];

export const PERMISSION_OPTIONS_ASSIGNED = [
  {
    label: 'Assigned',
    value: true
  },
  {
    label: 'Not Assigned',
    value: false
  }
];

export const PermissionAssignedCell: React.FC<{
  id: string;
  text: 'assigned' | 'included';
  assigned: boolean;
  onSelect: (params: { id: string; assigned: boolean }) => Promise<void>;
  children?: React.ReactNode;
}> = React.memo(({ id, text = 'included', assigned, onSelect, children }) => {
  return (
    <div className="flex items-center space-x-5">
      {children}
      <Select
        options={text === 'included' ? PERMISSION_OPTIONS_INCLUDED : PERMISSION_OPTIONS_ASSIGNED}
        value={assigned || false}
        popupMatchSelectWidth
        onSelect={(value) => {
          onSelect({ id, assigned: value });
        }}
        onClick={(e) => {
          e.preventDefault();
          e.stopPropagation();
        }}
      />
    </div>
  );
});

PermissionAssignedCell.displayName = 'PermissionAssignedCell';
