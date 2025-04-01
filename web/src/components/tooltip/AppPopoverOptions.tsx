import React, { useEffect, useRef, useState } from 'react';
import { AppPopover } from './AppPopover';
import { useClickAway, useMemoizedFn } from 'ahooks';
import isString from 'lodash/isString';
import { AppMaterialIcons } from '../icons';
import { Button, PopoverProps } from 'antd';
import { ButtonProps } from 'antd/lib';
import { createStyles } from 'antd-style';
import { useAntToken } from '@/styles/useAntToken';

export interface AppPopoverOption {
  key: string;
  onClick: () => void;
  label: string;
  description?: string;
  icon?: React.ReactNode;
}

const useStyles = createStyles(({ token, css }) => ({
  container: css`
    a {
      color: ${token.colorPrimary};
    }
  `,
  item: css`
    &:hover {
      background: ${token.controlItemBgHover};
    }
    &.selected {
      background: ${token.controlItemBgActive};
    }
  `,
  footer: {
    borderTop: `0.5px solid ${token.colorBorder}`
  }
}));

export const AppPopoverOptions: React.FC<{
  options: AppPopoverOption[];
  value?: string | AppPopoverOption;
  className?: string;
  children?: React.ReactNode;
  trigger?: 'click' | 'hover';
  showCheckIcon?: boolean;
  placement?: PopoverProps['placement'];
  footer?: React.ReactNode;
}> = ({
  trigger = 'click',
  children,
  options,
  value,
  showCheckIcon = true,
  placement = 'bottomRight',
  footer
}) => {
  const token = useAntToken();
  const { cx, styles } = useStyles();
  const [isOpenPopover, setIsOpenPopover] = useState(false);
  const rowClass = `cursor-pointer pr-2 pl-3 py-1 `;
  const selectedClass = `selected cursor-not-allowed`;
  const isSelected = useMemoizedFn((option: AppPopoverOption) => {
    if (!value) return false;
    if (isString(value)) return option.key === value;
    return option.key === (value as AppPopoverOption).key;
  });
  const useOpenClickTrigger = trigger === 'click';

  const optionsMap = options.map((option) => ({
    ...option,
    selected: isSelected(option)
  }));

  useEffect(() => {
    if (isOpenPopover) {
      const closePopover = () => {
        setIsOpenPopover(false);
      };
      setTimeout(() => {
        document.addEventListener('click', closePopover);
      }, 50);
      return () => {
        document.removeEventListener('click', closePopover);
      };
    }
  }, [isOpenPopover]);

  return (
    <AppPopover
      placement={placement}
      open={!useOpenClickTrigger ? undefined : isOpenPopover}
      destroyTooltipOnHide
      trigger={trigger}
      onOpenChange={(open) => {
        // isOpenPopoverRef.current = open;
      }}
      arrow={false}
      content={
        <div
          className={cx('flex flex-col space-y-1', styles.container)}
          style={{
            maxWidth: 280
          }}>
          <div className="flex flex-col space-y-1 p-1">
            {optionsMap.map((option) => (
              <div
                key={option.key}
                className={cx(
                  'transition',
                  styles.item,
                  rowClass,
                  `${option.selected ? selectedClass : ''}`,
                  `flex select-none items-center justify-between space-x-2 rounded p-1`
                )}
                onClick={() => {
                  option.onClick();
                  setIsOpenPopover(false);
                }}>
                <div
                  className={`flex h-full space-x-2 ${option.description ? 'items-start' : 'items-center'}`}>
                  <div
                    className="mt-0.5"
                    style={{
                      color: token.colorIcon
                    }}>
                    {option.icon}
                  </div>
                  <div
                    className={`flex flex-col space-y-0.5 ${option.description ? 'justify-start' : 'justify-center'}`}>
                    <div className="select-none text-base">{option.label}</div>
                    {option.description && (
                      <div className="select-none text-sm">{option.description}</div>
                    )}
                  </div>
                </div>

                {showCheckIcon && option.selected && (
                  <div className="">
                    <AppMaterialIcons icon="check" size={15} />
                  </div>
                )}
              </div>
            ))}
          </div>

          {footer && <div className={cx('pb-2 pt-2', styles.footer)}>{footer}</div>}
        </div>
      }>
      <div
        className="h-fit w-fit"
        onClick={() => {
          setIsOpenPopover(!isOpenPopover);
        }}>
        {children}
      </div>
    </AppPopover>
  );
};
