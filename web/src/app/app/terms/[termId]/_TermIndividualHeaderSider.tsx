import React from 'react';

import { AppMaterialIcons, AppTooltip } from '@/components';
import { Text } from '@/components';

export const TermIndividualHeaderSider: React.FC = () => {
  return (
    <div className="flex h-full w-full items-center justify-between">
      <Text>Details</Text>
      <div className="flex h-full items-center">
        <AppTooltip trigger={['click']} title="Edit">
          <Text className="flex !h-full cursor-pointer items-center">
            <AppMaterialIcons size={18} icon="help" />
          </Text>
        </AppTooltip>
      </div>
    </div>
  );
};
