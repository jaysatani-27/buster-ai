import { Text } from '@/components/text';
import { createStyles } from 'antd-style';
import React from 'react';

export const LabelAndInput: React.FC<{
  label: string;
  children: React.ReactNode;
}> = ({ label, children }) => {
  const { styles, cx } = useStyles();

  return (
    <div
      className={cx(
        'grid w-full grid-cols-[minmax(115px,115px)_1fr] items-center gap-2',
        styles.labelContainer
      )}>
      <Text size="sm" type="secondary">
        {label}
      </Text>

      {children}
    </div>
  );
};

const useStyles = createStyles(({ token }) => ({
  labelContainer: {
    height: `${token.controlHeight}px`
  }
}));
