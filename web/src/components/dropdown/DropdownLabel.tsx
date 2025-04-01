import React from 'react';
import { AppMaterialIcons, Text } from '@/components';
import { useAntToken } from '@/styles/useAntToken';

//antd.scss overrides label

export const DropdownLabel: React.FC<{ title: string; subtitle?: string }> = ({
  title,
  subtitle
}) => {
  const token = useAntToken();

  return (
    <div className="dropdown-label flex space-x-2 py-1">
      <div className="flex flex-col space-y-0.5">
        <Text className="!text-sm !font-normal">{title}</Text>
        {subtitle && (
          <Text className="subtitle !text-sm !font-normal" type="secondary">
            {subtitle}
          </Text>
        )}
      </div>

      <div
        className="check flex hidden w-full flex-col items-end justify-center"
        style={{
          color: token.colorIcon
        }}>
        <AppMaterialIcons icon="check" />
      </div>
    </div>
  );
};
