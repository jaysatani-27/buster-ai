import React, { useEffect } from 'react';
import { Button, Checkbox } from 'antd';
import { useAntToken } from '@/styles/useAntToken';
import { AppMaterialIcons } from '../icons';
import { Text } from '@/components';
import Link from 'next/link';

export const AppSelectItem: React.FC<{
  selected?: boolean;
  content: React.ReactNode;
  index?: number;
  disabled?: boolean;
  onClick?: () => void;
  hideCheckbox?: boolean;
  hideCheckmark?: boolean;
  link?: string;
}> = ({ onClick, link, disabled, hideCheckmark, hideCheckbox, index, selected, content }) => {
  const token = useAntToken();
  const validIndex = typeof index === 'number' ? true : false;

  useEffect(() => {
    if (typeof index !== 'number') return;

    const handleKeyPress = (event: KeyboardEvent) => {
      if (event.code === `Digit${index}`) {
        !disabled && onClick && onClick();
      }
    };

    document.addEventListener('keydown', handleKeyPress);

    return () => {
      document.removeEventListener('keydown', handleKeyPress);
    };
  }, []);

  return (
    <div
      onClick={(e) => {
        !disabled && onClick && onClick();
        e.stopPropagation();
        e.preventDefault();
      }}
      className="app-select-item group flex w-full items-center justify-start space-x-2">
      {!hideCheckbox && (
        <div
          className={`checkbox-container relative flex items-center transition group-hover:opacity-100 ${selected ? 'opacity-100' : 'opacity-0'}`}>
          <Checkbox
            checked={selected}
            disabled={disabled}
            onChange={(e) => {
              !disabled && onClick && onClick();
            }}
            onClick={(e) => {
              e.stopPropagation();
              e.preventDefault();
            }}
            style={{
              color: !disabled ? token.colorIcon : 'text-inherit'
            }}
          />
        </div>
      )}
      <div className="flex w-full items-center justify-between space-x-1">
        <Text className={`w-full ${link ? 'mr-5' : ''}`}>{content}</Text>

        {(validIndex || (!hideCheckmark && validIndex && hideCheckbox && selected)) && (
          <div className="flex items-center space-x-1">
            {!hideCheckmark && validIndex && hideCheckbox && selected && (
              <AppMaterialIcons
                className={!disabled ? '' : '!text-inherit'}
                style={{
                  color: !disabled ? token.colorIcon : 'text-inherit'
                }}
                icon="check"
              />
            )}
            {validIndex && (
              <Text
                type={disabled ? 'tertiary' : 'secondary'}
                className={`min-w-[9px] text-center ${link ? '' : ''} `}>
                {index}
              </Text>
            )}
          </div>
        )}

        {link && (
          <div
            className="absolute opacity-0 group-hover:opacity-100"
            style={{
              marginRight: 0,
              right: 2
            }}
            onClick={(e) => {
              e.stopPropagation();
              e.preventDefault();
            }}>
            <Link href={link}>
              <Button
                className="flex"
                type="text"
                icon={<AppMaterialIcons icon="arrow_right_alt" size={14} />}
              />
            </Link>
          </div>
        )}
      </div>
    </div>
  );
};
