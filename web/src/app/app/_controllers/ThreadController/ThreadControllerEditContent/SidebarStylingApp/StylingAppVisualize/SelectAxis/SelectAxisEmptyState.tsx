import React from 'react';
import { Text } from '@/components/text';

export const SelectAxisEmptyState: React.FC = () => {
  return (
    <div className="flex h-full min-h-12 items-center justify-center p-3">
      <Text type="tertiary">No data to display</Text>
    </div>
  );
};
