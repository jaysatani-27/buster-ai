'use client';

import React, { useMemo } from 'react';
import { useFontStyles } from './useFontStyles';
import { createStyles } from 'antd-style';
import tailwind from '../../../tailwind.config';
import { AppTooltip } from '../tooltip';
const sizes = tailwind.theme.fontSize;

interface TextProps {
  type?: 'secondary' | 'tertiary' | 'default' | 'danger' | 'primary' | 'inherit' | 'link';
  size?: 'xs' | 'sm' | 'base' | 'md' | 'lg' | '2xl' | 'xxs';
  ellipsis?: boolean | { tooltip: boolean | string };
  className?: string;
  style?: React.CSSProperties;
  onClick?: React.MouseEventHandler<HTMLSpanElement>;
  children: React.ReactNode;
  lineHeight?: number;
}

const useTextStyles = createStyles(({ css, token }) => {
  return {
    '2xl': css`
      font-size: ${sizes['2xl']};
    `,
    lg: css`
      font-size: ${sizes.lg};
    `,
    md: css`
      font-size: ${sizes.md};
    `,
    sm: css`
      font-size: ${sizes.sm};
    `,
    base: css`
      font-size: ${sizes.base};
    `,
    xs: css`
      font-size: ${sizes.xs};
    `,
    xxs: css`
      font-size: ${sizes.xxs};
    `
  };
});

export const Text = React.memo<
  {
    children?: React.ReactNode;
  } & TextProps
>(({ children, ...props }) => {
  return (
    <TextWrapper {...props}>
      <TextComponent {...props}>{children}</TextComponent>
    </TextWrapper>
  );
});
Text.displayName = 'Text';

const TextWrapper: React.FC<
  {
    children?: React.ReactNode;
  } & TextProps
> = React.memo(({ children, ellipsis }) => {
  const tooltip = useMemo(() => {
    if (ellipsis && typeof ellipsis === 'object' && 'tooltip' in ellipsis) {
      return ellipsis.tooltip;
    }
  }, [ellipsis]);

  const title = tooltip ? (typeof tooltip === 'string' ? tooltip : (children as string)) : '';

  return tooltip ? (
    <AppTooltip performant title={title}>
      {children}
    </AppTooltip>
  ) : (
    <>{children}</>
  );
});
TextWrapper.displayName = 'TextWrapper';

const TextComponent = React.memo<TextProps>(
  ({ children, lineHeight, size = 'base', type = 'default', ellipsis, ...props }) => {
    const { cx, styles } = useFontStyles();
    const { styles: styles2 } = useTextStyles();

    const colorClass = useMemo(() => {
      if (type === 'default') return styles.default;
      if (type === 'secondary') return styles.secondary;
      if (type === 'tertiary') return styles.tertiary;
      if (type === 'primary') return styles.primary;
      if (type === 'danger') return styles.danger;
      if (type === 'link') return styles.link;
      return '';
    }, [type]);

    const sizeClass = useMemo(() => {
      if (size === 'xs') return styles2.xs;
      if (size === 'sm') return styles2.sm;
      if (size === 'md') return styles2.md;
      if (size === 'lg') return styles2.lg;
      if (size === '2xl') return styles2['2xl'];
      if (size === 'xxs') return styles2.xxs;
      if (size === 'base') return styles2.base;
      else return styles2.base;
    }, [size]);

    const memoizedStyles = useMemo(() => {
      return {
        lineHeight: lineHeight ? `${lineHeight}px` : undefined,
        cursor: type === 'link' ? 'pointer' : undefined,
        ...props.style
      };
    }, [lineHeight, type]);

    return (
      <span
        {...props}
        className={cx(size, props.className, colorClass, sizeClass, ellipsis && styles.ellipsis)}
        style={memoizedStyles}>
        {children}
      </span>
    );
  }
);
TextComponent.displayName = 'TextComponent';
