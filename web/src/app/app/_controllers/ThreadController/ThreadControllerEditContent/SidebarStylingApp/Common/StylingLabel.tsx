import React from 'react';
import { Text } from '@/components/text';

interface StylingLabelProps {
  label: string;
  labelExtra?: React.ReactNode;
  children: React.ReactNode;
  className?: string;
}

export const StylingLabel: React.FC<StylingLabelProps> = ({
  label,
  labelExtra,
  children,
  className = ''
}) => {
  return (
    <div className={`flex flex-col space-y-2.5 ${className}`}>
      <div className="flex items-center justify-between">
        <Text size="sm" type="secondary">
          {label}
        </Text>
        {labelExtra}
      </div>
      {children}
    </div>
  );
};
