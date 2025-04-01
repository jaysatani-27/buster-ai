import { useAntToken } from '@/styles/useAntToken';
import { ConfigProvider, Menu, MenuProps } from 'antd';
import { createStyles } from 'antd-style';
import { AnimatePresence, motion } from 'framer-motion';
import React, { forwardRef, useMemo } from 'react';
import { BusterListContextMenu, BusterMenuItemType } from './interfaces';
import { ItemType, MenuItemType } from 'antd/es/menu/interface';

export interface BusterListContentMenuProps {
  menu: BusterListContextMenu;
  open: boolean;
  placement: { x: number; y: number };
  id: string;
}

export const BusterListContentMenu = forwardRef<HTMLDivElement, BusterListContentMenuProps>(
  ({ menu, open, placement, id }, ref) => {
    const { styles, cx } = useStyles();

    const computedItems: MenuProps['items'] = useMemo(
      () =>
        menu.items?.map((item) => {
          return {
            ...(item as MenuItemType),
            popupClassName: cx(styles.popUpClassMenu),
            onClick: () => {
              (item as BusterMenuItemType)?.onClick?.(id);
            }
          };
        }) || [],
      [menu.items, id]
    );

    return (
      <ConfigProvider
        theme={{
          components: {
            Menu: {
              itemMarginBlock: 4,
              itemMarginInline: 4,
              iconMarginInlineEnd: 8
            }
          }
        }}>
        <AnimatePresence mode="wait">
          {open && (
            <motion.div
              ref={ref}
              initial={{ opacity: 0, y: 0, scaleY: 0.9 }}
              animate={{ opacity: 1, y: 0, scaleY: 1 }}
              exit={{ opacity: 0, y: 0, scaleY: 0.9 }}
              transition={{ duration: 0.1 }}
              className={cx(styles.contextMenu, 'list-context-menu fixed')}
              style={{
                top: placement.y,
                left: placement.x,
                minHeight: 10,
                minWidth: 175,
                padding: 0
              }}>
              <Menu {...menu} items={computedItems} mode="vertical" />
            </motion.div>
          )}
        </AnimatePresence>
      </ConfigProvider>
    );
  }
);
BusterListContentMenu.displayName = 'BusterListContentMenu';

const useStyles = createStyles(({ token, prefixCls, css }) => ({
  popUpClassMenu: css`
    .busterv2-menu {
      border: 0.5px solid ${token.colorBorder};
    }

    .busterv2-menu-submenu-title {
      display: flex !important;
      align-items: center;
    }
  `,
  contextMenu: css`
    box-shadow: ${token.boxShadowSecondary};
    border-radius: ${token.borderRadiusLG};
    background-color: ${token.colorBgElevated};
    border: 0.5px solid ${token.colorBorder};
  `
}));
