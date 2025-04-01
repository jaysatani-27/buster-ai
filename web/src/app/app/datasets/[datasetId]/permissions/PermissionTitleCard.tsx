'use client';

import React from 'react';
import { Title, Text } from '@/components/text';

export const PermissionTitleCard: React.FC<{}> = React.memo(({}) => {
  return (
    <div className="flex flex-col gap-1.5">
      <Title level={3}>Dataset Permissions</Title>
      <Text style={{ fontSize: '15px' }} type="secondary">
        Manage who can build dashboards & metrics using this dataset
      </Text>
    </div>
  );
});

PermissionTitleCard.displayName = 'PermissionTitleCard';
