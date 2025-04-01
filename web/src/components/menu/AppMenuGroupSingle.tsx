import { Collapse, ConfigProvider, Menu, CollapseProps } from 'antd';
import React, { PropsWithChildren } from 'react';
import { createStyles } from 'antd-style';
import { ItemType, MenuItemType } from 'antd/es/menu/interface';
import { MenuProps } from 'antd/lib';
import { ExpandIcon } from './ExpandIcon';
import { useMemoizedFn } from 'ahooks';

export const useMenuGroupStyles = createStyles(({ token, css }) => ({
  container: css`
    .busterv2-collapse-header {
      color: ${token.colorIcon} !important;
      align-items: center !important;
      display: flex !important;
      transition: background 0.3s;
      border-radius: ${token.borderRadius}px !important;
      &:hover {
        background: ${//@ts-ignore
        token.Menu?.itemHoverBg} !important;
      }
    }
    .busterv2-collapse-header-text {
      flex: none !important;
      margin-inline-end: inherit !important;
      align-items: center;
    }
    .busterv2-splitter-menu-left {
      border-left: 2px solid ${token.colorBorder} !important;
      margin-left: 8px;
      padding-left: 8px;
    }
    .busterv2-collapse-expand-icon {
      padding-left: 0 !important;
    }
    .busterv2-menu-submenu-title {
      padding-right: 6px !important;
    }
    .busterv2-menu-title-content {
      display: inherit !important;
    }
  `,
  menu: css`
    a {
      overflow: hidden;
      text-overflow: ellipsis;
    }
  `
}));

export const menuToken = {
  components: {
    Collapse: {
      contentPadding: '0px 0px',
      headerPadding: '1.5px 4px',
      paddingXXS: 0,
      headerBg: 'transparent',
      contentBg: 'transparent',
      marginSM: 8
    }
  }
};

const DEFAULT_KEY = '1';

export const AppMenuGroupSingle: React.FC<
  PropsWithChildren<{
    label: string;
    items: ItemType<MenuItemType>[];
    selectedKey: string;
    onOpenChange?: MenuProps['onOpenChange'];
  }>
> = React.memo(({ onOpenChange, items, label, selectedKey }) => {
  const { styles, cx } = useMenuGroupStyles();

  const menuItems: CollapseProps['items'] = [
    {
      key: DEFAULT_KEY,
      label: <span className="select-none !text-sm">{label}</span>,
      children: (
        <Menu
          className={cx(styles.menu, 'overflow-hidden')}
          expandIcon={(v) => <ExpandIcon {...v} />}
          inlineIndent={6}
          selectable
          selectedKeys={[selectedKey]}
          mode="inline"
          items={items}
          onOpenChange={onOpenChange}
        />
      )
    }
  ];

  const expandIcon = useMemoizedFn((v) => {
    return <ExpandIcon {...v} />;
  });

  return (
    <ConfigProvider theme={menuToken}>
      <Collapse
        destroyInactivePanel
        rootClassName={cx(styles.container, 'overflow-hidden')}
        defaultActiveKey={[DEFAULT_KEY]}
        expandIconPosition={'end'}
        items={menuItems}
        bordered={false}
        expandIcon={expandIcon}
      />
    </ConfigProvider>
  );
});

AppMenuGroupSingle.displayName = 'AppMenuGroupSingle';
