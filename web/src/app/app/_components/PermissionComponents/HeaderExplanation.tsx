import React from 'react';
import { Title, Text } from '@/components';

export const HeaderExplanation: React.FC<{
  className?: string;
  title?: string;
  description?: string;
}> = React.memo(
  ({
    className = '',
    title = 'Access & lineage',
    description = 'View which users can query this dataset. Lineage is provided to show where each user’s access originates from.'
  }) => {
    return (
      <div className={`flex flex-col space-y-1.5 ${className}`}>
        <Title level={4}>{title}</Title>
        <Text type="secondary">{description}</Text>
      </div>
    );
  }
);
HeaderExplanation.displayName = 'HeaderExplanation';
