import { TeamRole } from '@/api';
import { Select } from 'antd';
import React from 'react';

export const TEAM_ROLE_OPTIONS: { label: string; value: TeamRole }[] = [
  {
    label: 'Manager',
    value: TeamRole.MANAGER
  },
  {
    label: 'Member',
    value: TeamRole.MEMBER
  },
  {
    label: 'Not a Member',
    value: TeamRole.NONE
  }
];

export const PermissionAssignTeamRole: React.FC<{
  role: TeamRole;
  id: string;
  onRoleChange: (data: { id: string; role: TeamRole }) => void;
  children?: React.ReactNode;
}> = React.memo(({ role, id, onRoleChange, children }) => {
  return (
    <div className="flex items-center space-x-5">
      {children}
      <Select
        options={TEAM_ROLE_OPTIONS}
        value={role}
        onChange={(v) => {
          onRoleChange({ id, role: v });
        }}
        onClick={(e) => {
          e.stopPropagation();
          e.preventDefault();
        }}
      />
    </div>
  );
});

PermissionAssignTeamRole.displayName = 'PermissionAssignTeamRole';
