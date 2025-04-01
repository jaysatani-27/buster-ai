import React from 'react';
import { SelectAxisContainerId } from '../config';
import { ZoneIdToTitle } from '../helper';
import { Text } from '@/components/text';
import { Divider } from 'antd';
import { zoneIdToAxisSettingContent } from './config';

export const SelectAxisSettingContent: React.FC<{ zoneId: SelectAxisContainerId }> = ({
  zoneId
}) => {
  const SettingContentComponent = zoneIdToAxisSettingContent[zoneId];

  return (
    <div className="flex min-w-[305px] max-w-[305px] flex-col">
      <TitleContent zoneId={zoneId} />

      {SettingContentComponent && (
        <>
          <Divider />

          <div className="flex flex-col space-y-2 p-3">
            <SettingContentComponent zoneId={zoneId} />
          </div>
        </>
      )}
    </div>
  );
};

const TitleContent: React.FC<{ zoneId: SelectAxisContainerId }> = ({ zoneId }) => {
  return (
    <div className="px-3 py-2.5">
      <Text>{ZoneIdToTitle[zoneId]}</Text>
    </div>
  );
};
