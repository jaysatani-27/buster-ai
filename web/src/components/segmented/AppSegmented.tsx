import React, { useMemo } from 'react';
import { ConfigProvider, Segmented, SegmentedProps, ThemeConfig } from 'antd';
import { createStyles } from 'antd-style';
import { busterAppStyleConfig } from '@/styles/busterAntDStyleConfig';
import Link from 'next/link';
import { useMemoizedFn } from 'ahooks';
import { useRouter } from 'next/navigation';
import { AppTooltip } from '@/components/tooltip';
const token = busterAppStyleConfig.token!;

type SegmentedOption = {
  value: string;
  label?: string;
  link?: string;
  onHover?: () => void;
  icon?: React.ReactNode;
  tooltip?: string;
};
export interface AppSegmentedProps extends Omit<SegmentedProps, 'options'> {
  bordered?: boolean;
  options: SegmentedOption[];
}

const useStyles = createStyles(({ css, token }) => {
  return {
    segmented: css``
  };
});

const THEME_CONFIG: ThemeConfig = {
  components: {
    Segmented: {
      itemColor: token.colorTextDescription,
      trackBg: 'transparent',
      itemSelectedColor: token.colorTextBase,
      itemSelectedBg: token.controlItemBgActive,
      colorBorder: token.colorBorder,
      boxShadowTertiary: 'none'
    }
  }
};

export const AppSegmented = React.memo<AppSegmentedProps>(
  ({ size = 'small', bordered = true, onChange, options: optionsProps, ...props }) => {
    const { cx, styles } = useStyles();
    const router = useRouter();

    const options = useMemo(() => {
      return optionsProps.map((option) => ({
        value: option.value,
        label: <SegmentedItem option={option} />
      }));
    }, [optionsProps]);

    const onChangePreflight = useMemoizedFn((value: string | number) => {
      const link = optionsProps.find((option) => option.value === value)?.link;
      if (link) {
        router.push(link);
      }
      onChange?.(value);
    });

    return (
      <ConfigProvider theme={THEME_CONFIG}>
        <Segmented
          {...props}
          onChange={onChangePreflight}
          options={options}
          size={size}
          className={cx(
            styles.segmented,
            props.className,
            '!shadow-none',
            !bordered && 'no-border'
          )}
        />
      </ConfigProvider>
    );
  }
);
AppSegmented.displayName = 'AppSegmented';

const SegmentedItem: React.FC<{ option: SegmentedOption }> = ({ option }) => {
  return (
    <AppTooltip mouseEnterDelay={0.75} title={option.tooltip}>
      <SegmentedItemLink href={option.link}>
        <div className="flex items-center gap-0.5" onClick={option.onHover}>
          {option.icon}
          {option.label}
        </div>
      </SegmentedItemLink>
    </AppTooltip>
  );
};

const SegmentedItemLink: React.FC<{ href?: string; children: React.ReactNode }> = ({
  href,
  children
}) => {
  if (!href) return children;
  return (
    <Link prefetch={true} href={href}>
      {children}
    </Link>
  );
};
