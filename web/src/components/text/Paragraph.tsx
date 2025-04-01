'use client';

import React from 'react';
import type { TextProps as AntDTextProps } from 'antd/es/typography/Text';
import { useFontStyles } from './useFontStyles';
import { Typography } from 'antd';

const { Paragraph: AntParagraph } = Typography;

interface ParagraphProps extends Omit<AntDTextProps, 'type'> {
  type?: 'secondary' | 'tertiary' | 'default';
}

export const Paragraph = React.memo<
  {
    children: React.ReactNode;
  } & ParagraphProps
>(({ children, type = 'default', ...props }) => {
  const { cx, styles } = useFontStyles();

  return (
    <AntParagraph
      {...props}
      className={cx(
        'busterv2-paragraph',
        props.className,
        type === 'default' && styles.default,
        type === 'secondary' && styles.secondary,
        type === 'tertiary' && styles.tertiary
      )}>
      {children}
    </AntParagraph>
  );
});
Paragraph.displayName = 'Paragraph';
