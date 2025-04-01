import { OrganizationUser } from '@/api/buster_rest';

export const OrganizationUserStatusText: Record<OrganizationUser['status'], string> = {
  active: 'Active',
  inactive: 'Inactive'
};

export const OrganizationUserRoleText: Record<OrganizationUser['role'], string> = {
  dataAdmin: 'Data Admin',
  workspaceAdmin: 'Workspace Admin',
  querier: 'Querier',
  restrictedQuerier: 'Restricted Querier',
  viewer: 'Viewer'
};
