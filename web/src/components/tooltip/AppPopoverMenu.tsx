import React, { useLayoutEffect, useMemo } from 'react';
import { Divider, Dropdown, MenuProps, DropdownProps, Input } from 'antd';
import { useAntToken } from '@/styles/useAntToken';
import { createStyles } from 'antd-style';
import { AppSelectItem } from '../select/AppSelectItem';
import { useMemoizedFn, useMount } from 'ahooks';

type Item = {
  label: React.ReactNode;
  key?: string;
  index?: number;
  onClick?: () => void;
  icon?: React.ReactNode;
  disabled?: boolean;
  link?: string;
};

export interface AppPopoverMenuProps extends DropdownProps {
  headerContent?: React.ReactNode | string;
  children: React.ReactNode;
  footerContent?: React.ReactNode;
  items?: Item[];
  contentWidth?: number;
  selectedItems?: string[];
  hideCheckbox?: boolean;
  doNotSortSelected?: boolean;
  disabled?: boolean;
}

const useAppPopoverMenuStyles = createStyles(({ css, token }) => ({
  container: css`
    .busterv2-dropdown-menu-item {
      &:hover {
        .checkbox-container {
          opacity: 1 !important;
        }
      }
    }
  `
}));

export const AppPopoverMenu: React.FC<AppPopoverMenuProps> = React.memo(
  ({
    items,
    children,
    headerContent,
    footerContent,
    contentWidth = 205,
    selectedItems = [],
    hideCheckbox = false,
    destroyPopupOnHide = true,
    doNotSortSelected = false,
    disabled = false,
    ...props
  }) => {
    const token = useAntToken();
    const { styles, cx } = useAppPopoverMenuStyles();
    const [filterText, setFilterText] = React.useState('');

    const footerContentContainer = !!footerContent ? (
      <div className="px-2 py-1.5">{footerContent}</div>
    ) : null;

    const filterItems = useMemoizedFn((items: Item[]) => {
      const filterTextLowerCase = filterText.toLowerCase();
      return items.filter((item) => {
        if (filterText === '') return true;
        const keyLowerCase = item.key?.toString().toLowerCase();
        const labelLowerCase = item.label?.toString().toLowerCase();
        return (
          keyLowerCase?.includes(filterTextLowerCase) ||
          labelLowerCase?.includes(filterTextLowerCase)
        );
      });
    });

    const _selectedItems = useMemo(
      () =>
        filterItems(
          doNotSortSelected
            ? []
            : items?.filter((item) => selectedItems.includes(item.key as string)) || []
        ),
      [doNotSortSelected, filterText, items, selectedItems]
    );
    const notSelectedItems = useMemo(
      () =>
        filterItems(
          doNotSortSelected
            ? items || []
            : items?.filter((item) => !selectedItems.includes(item.key as string)) || []
        ),
      [doNotSortSelected, filterText, items, selectedItems]
    );

    const createItem = useMemoizedFn((item: Item, index: number) => ({
      disabled: item.disabled,
      icon: item.icon,
      index,
      label: (
        <AppSelectItem
          disabled={item.disabled}
          index={item.index}
          content={item?.label}
          hideCheckbox={hideCheckbox}
          link={item.link}
          onClick={() => {
            item.onClick && item.onClick();
          }}
          selected={selectedItems.includes(item.key as string)}
        />
      ),
      key: (item?.key as string) || (index.toString() as string)
    }));

    const selectedItemsInternal: Item[] = useMemo(
      () => _selectedItems.map(createItem),
      [_selectedItems, createItem]
    );
    const internalItems: MenuProps['items'] = useMemo(
      () => notSelectedItems.map(createItem),
      [notSelectedItems, createItem]
    );

    const dropdownRender = useMemoizedFn((menu: React.ReactNode) => {
      return (
        <div
          style={{
            minWidth: contentWidth,
            backgroundColor: token.colorBgElevated,
            borderRadius: token.borderRadiusLG,
            boxShadow: token.boxShadowSecondary
          }}>
          <HeaderContentContainer
            headerContent={headerContent}
            filterText={filterText}
            setFilterText={setFilterText}
          />

          {!!headerContent && <Divider />}

          {!!selectedItemsInternal.length && (
            <>
              <div className="p-1">
                {selectedItemsInternal.map((item, index) => (
                  <SelectedItem {...item} key={String(index)} />
                ))}
              </div>

              {!!internalItems.length && <Divider />}
            </>
          )}

          {!!menu &&
            React.cloneElement(menu as React.ReactElement, {
              style: {
                boxShadow: 'none'
              }
            })}

          {!!footerContentContainer && !!items?.length && <Divider />}

          {footerContentContainer}
        </div>
      );
    });

    const memoizedMenu = useMemo(() => {
      return {
        className: cx(styles.container),
        rootClassName: '',
        items: internalItems,
        selectable: true,
        selectedKeys: selectedItems
      };
    }, [internalItems, selectedItems]);

    useLayoutEffect(() => {
      if (!props.open) {
        setFilterText('');
      }
    }, [props.open]);

    return (
      <Dropdown
        {...props}
        disabled={disabled}
        destroyPopupOnHide={destroyPopupOnHide}
        open={props.open}
        menu={memoizedMenu}
        dropdownRender={dropdownRender}>
        {children}
      </Dropdown>
    );
  }
);

AppPopoverMenu.displayName = 'AppPopoverMenu';
const useSelectedItemStyles = createStyles(({ css, token }) => ({
  container: css`
    cursor: pointer;
    display: flex;
    items-center: center;
    height: 28px;
    padding: 0 0 0 12px;
    border-radius: ${token.borderRadius}px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    &:hover {
      background: ${token.controlItemBgHover};
    }
  `,
  icon: {
    color: token.colorIcon,
    fontSize: token.fontSizeIcon
  }
}));

const SelectedItem: React.FC<Omit<Item, 'key'>> = ({ label, onClick, icon }) => {
  const { styles, cx } = useSelectedItemStyles();

  return (
    <div onClick={onClick} className={cx(styles.container, 'relative w-full')}>
      {icon && <span className={cx(styles.icon, 'mr-2 flex items-center')}>{icon}</span>}
      {label}
    </div>
  );
};

const HeaderContentContainer: React.FC<{
  headerContent?: React.ReactNode | string;
  filterText: string;
  setFilterText: (text: string) => void;
}> = ({ headerContent, setFilterText, filterText }) => {
  const token = useAntToken();

  const onChange = useMemoizedFn((e: React.ChangeEvent<HTMLInputElement>) => {
    e.stopPropagation();
    e.preventDefault();
    setFilterText(e.target.value);
  });

  if (!headerContent) return;

  const headerIsText = typeof headerContent === 'string';

  return (
    <div
      className="flex items-center px-3 py-1"
      style={{
        color: token.colorTextDescription,
        height: 38
      }}>
      {headerIsText ? (
        <Input
          className="!pl-0"
          variant="borderless"
          placeholder={headerContent as string}
          value={filterText}
          onChange={onChange}
        />
      ) : (
        headerContent
      )}
    </div>
  );
};
