import React from 'react';
import CircleSpinnerLoader from './CircleSpinnerLoader';
import { Text } from '@/components';

export const CircleSpinnerLoaderContainer: React.FC<{
  text?: string;
  className?: string;
}> = ({ className = '', text = '' }) => {
  return (
    <div className={`flex h-full w-full flex-col items-center justify-center ${className}`}>
      <CircleSpinnerLoader />
      {text && <Text className="mt-3">{text}</Text>}
    </div>
  );
};
