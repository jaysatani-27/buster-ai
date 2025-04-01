import React, { PropsWithChildren, useState } from 'react';
import { Popover, PopoverProps } from 'antd';
import { createStyles } from 'antd-style';

interface Props extends PopoverProps {
  headerContent?: React.ReactNode;
  content: React.ReactNode;
  performant?: boolean;
}

const useStyles = createStyles(({ token, css }) => ({
  header: css`
    border-bottom: 0.5px solid ${token.colorBorder};
    padding-top: 6px;
    padding-bottom: 6px;
    padding-left: 6px;
    padding-right: 6px;
  `,
  popover: css`
    .busterv2-popover-inner {
      padding: 0 !important;
    }
  `
}));

export const AppPopover: React.FC<PropsWithChildren<Props>> = ({
  children,
  content,
  headerContent,
  arrow = false,
  className,
  performant,
  destroyTooltipOnHide = true,
  ...props
}) => {
  const [performatMounted, setPerformantMounted] = useState(false);
  const { styles, cx } = useStyles();

  const contentWithHeader = headerContent ? (
    <div className="flex w-full flex-col">
      {headerContent && <div className={cx(styles.header)}>{headerContent}</div>}
      {content}
    </div>
  ) : (
    content
  );

  if (performant && !performatMounted) {
    return (
      <span className="flex" onMouseEnter={() => setPerformantMounted(true)}>
        {children}
      </span>
    );
  }

  return (
    <Popover
      {...props}
      destroyTooltipOnHide={destroyTooltipOnHide}
      overlayClassName={cx(styles.popover, className, '')}
      arrow={arrow}
      content={contentWithHeader}>
      {performant === undefined ? children : <span className="flex">{children}</span>}
    </Popover>
  );
};
