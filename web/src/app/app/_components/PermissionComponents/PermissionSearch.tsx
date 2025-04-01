import React from 'react';
import { Input } from 'antd';
import { AppMaterialIcons } from '@/components';
import { useMemoizedFn } from 'ahooks';

export const PermissionSearch: React.FC<{
  className?: string;
  searchText: string;
  setSearchText: (text: string) => void;
  placeholder?: string;
}> = ({ className = '', searchText, setSearchText, placeholder = 'Search by name or email' }) => {
  const onChange = useMemoizedFn((e: React.ChangeEvent<HTMLInputElement>) => {
    setSearchText(e.target.value);
  });

  return (
    <div className={`flex w-full flex-col space-y-1.5 ${className}`}>
      <Input
        className="w-[280px] max-w-[280px]"
        placeholder={placeholder}
        value={searchText}
        onChange={onChange}
        allowClear
        prefix={<AppMaterialIcons icon={'search'} />}
      />
    </div>
  );
};
PermissionSearch.displayName = 'PermissionOverviewSearch';
