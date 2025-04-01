import { Collapse, ConfigProvider, Menu, CollapseProps, Button } from 'antd';
import React, { PropsWithChildren, useMemo, useState } from 'react';
import { ItemType, MenuItemType } from 'antd/es/menu/interface';
import { MenuProps } from 'antd/lib';
import { menuToken, useMenuGroupStyles } from './AppMenuGroupSingle';
import {
  DndContext,
  closestCenter,
  PointerSensor,
  useSensor,
  useSensors,
  DragEndEvent,
  DragOverlay,
  MouseSensor,
  TouchSensor,
  MeasuringStrategy,
  DragOverEvent,
  DragStartEvent
} from '@dnd-kit/core';
import { arrayMove, useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { useAntToken } from '@/styles/useAntToken';
import { ExpandIcon } from './ExpandIcon';
import { useMemoizedFn } from 'ahooks';
import { AppMaterialIcons } from '../icons';
import { createStyles } from 'antd-style';

const DEFAULT_KEY = '1';

export const AppMenuGroupSingleSortable: React.FC<
  PropsWithChildren<{
    label: string;
    items: (ItemType<MenuItemType> & { value: string })[];
    selectedKey: string;
    onDelete?: (id: string) => void;
    onOpenChange?: MenuProps['onOpenChange'];
    onChangeOrder: (
      items: {
        value: string;
      }[]
    ) => void;
  }>
> = React.memo(({ onChangeOrder, onOpenChange, items, label, selectedKey, onDelete }) => {
  const { styles, cx } = useMenuGroupStyles();
  const [activeId, setActiveId] = useState<string | null>(null);
  const activeIndex = activeId ? items.findIndex((i) => i?.key === activeId) : -1;
  const activeDragItem = items[activeIndex] as MenuItemType;

  const sortItems = useMemo(() => {
    return items.map((item, index) => {
      if (item?.type === 'item' || (item as MenuItemType).label) {
        let _item = item as MenuItemType;
        return {
          ...item,
          icon: <></>,
          className: 'relative',
          label: (
            <SortableItem
              index={index}
              activeIndex={activeIndex}
              icon={_item.icon}
              id={_item.key as string}
              onDelete={onDelete}
              isLast={items.length - 1 === index}>
              {_item.label}
            </SortableItem>
          )
        } as MenuItemType;
      }
      return { ...item, itemIcon: (item as MenuItemType).icon } as MenuItemType;
    });
  }, [items]);

  const menuItems: CollapseProps['items'] = useMemo(() => {
    return [
      {
        key: DEFAULT_KEY,
        label,
        children: (
          <Menu
            className={cx(styles.menu)}
            expandIcon={(v) => <ExpandIcon {...v} />}
            inlineIndent={6}
            selectable
            mode="inline"
            items={sortItems}
            selectedKeys={[selectedKey]}
            onOpenChange={onOpenChange}
          />
        )
      }
    ];
  }, [sortItems, label, selectedKey]);

  const onChange = useMemoizedFn((key: string | string[]) => {
    //
  });

  const handleDragEnd = useMemoizedFn((event: DragEndEvent) => {
    // setDraggingOverId(null);
    setActiveId(null);
    const { active, over } = event;
    if (active.id !== over?.id) {
      const newItems = arrayMove(
        items,
        activeIndex,
        items.findIndex((i) => i?.key === over?.id)
      );
      onChangeOrder(newItems);
    }
  });

  const onDragOver = useMemoizedFn((event: DragOverEvent) => {
    // setDraggingOverId(event?.over?.id as string);
  });

  const onDragStart = useMemoizedFn((event: DragStartEvent) => {
    setActiveId(event?.active?.id as string);
  });

  return (
    <MemoizedDragContext
      handleDragEnd={handleDragEnd}
      onDragOver={onDragOver}
      onDragStart={onDragStart}>
      <Collapse
        destroyInactivePanel
        rootClassName={cx(styles.container, '')}
        defaultActiveKey={[DEFAULT_KEY]}
        onChange={onChange}
        expandIconPosition={'end'}
        items={menuItems}
        bordered={false}
        expandIcon={ExpandIcon}
      />

      <DragOverlay dropAnimation={null}>
        {activeId && activeDragItem && (
          <MenuItem
            icon={activeDragItem.icon as React.ReactNode}
            label={activeDragItem.label}
            onClick={activeDragItem.onClick as () => void}
          />
        )}
      </DragOverlay>
    </MemoizedDragContext>
  );
});
AppMenuGroupSingleSortable.displayName = 'AppMenuGroupSingleSortable';

const MemoizedDragContext = React.memo(
  ({
    children,
    handleDragEnd,
    onDragOver,
    onDragStart
  }: {
    children: React.ReactNode;
    handleDragEnd: (event: DragEndEvent) => void;
    onDragOver: (event: DragOverEvent) => void;
    onDragStart: (event: DragStartEvent) => void;
  }) => {
    const sensors = useSensors(
      useSensor(MouseSensor, {
        activationConstraint: {
          distance: 5
        }
      }),
      useSensor(TouchSensor, {
        activationConstraint: {
          distance: 5
        }
      })
    );

    const measuring = useMemo(
      () => ({
        droppable: {
          strategy: MeasuringStrategy.Always
        }
      }),
      []
    );

    return (
      <ConfigProvider theme={menuToken}>
        <DndContext
          sensors={sensors}
          measuring={measuring}
          collisionDetection={closestCenter}
          onDragEnd={handleDragEnd}
          onDragOver={onDragOver}
          onDragStart={onDragStart}>
          {children}
        </DndContext>
      </ConfigProvider>
    );
  }
);
MemoizedDragContext.displayName = 'MemoizedDragContext';

const useStyles = createStyles(({ token, css }) => ({
  sortableItemHoverTop: css`
    &.isOver {
      background: ${token.colorPrimary};
    }
  `,
  icon: css`
    color: ${token.colorIcon};
  `
}));

const SortableItem: React.FC<{
  isLast: boolean;
  id: string;
  icon?: React.ReactNode;
  children: React.ReactNode;
  index: number;
  activeIndex: number;
  onDelete?: (id: string) => void;
}> = React.memo(({ id, icon, index, activeIndex, isLast, children, onDelete }) => {
  const { styles, cx } = useStyles();
  const { attributes, isOver, listeners, setNodeRef, transform, transition } = useSortable({
    id: id
  });
  const style: React.CSSProperties = useMemo(
    () => ({
      transform: CSS.Transform.toString(transform),
      transition
    }),
    [transform, transition]
  );

  return (
    <>
      <div
        className={cx(
          'absolute left-0 right-0 w-full',
          styles.sortableItemHoverTop,
          isOver ? 'opacity-100' : 'opacity-0',
          {
            isOver: isOver
          }
        )}
        style={{
          height: 1,
          bottom: activeIndex === 0 ? 0 : undefined
        }}
      />

      <div
        className="group flex w-full items-center justify-between space-x-2 overflow-hidden"
        ref={setNodeRef}
        style={style}
        {...attributes}
        {...listeners}>
        <div className="flex w-full items-center space-x-2.5 overflow-hidden">
          {icon && <div className={cx(styles.icon, 'flex items-center')}>{icon}</div>}
          {children}
        </div>

        {onDelete && (
          <Button
            onClick={() => onDelete(id)}
            className={cx(styles.icon, '!hidden cursor-pointer group-hover:!flex')}
            icon={<AppMaterialIcons icon="close" />}
            type="text"
          />
        )}
      </div>
    </>
  );
});
SortableItem.displayName = 'SortableItem';

const MenuItem: React.FC<{
  icon: React.ReactNode | undefined;
  label: React.ReactNode;
  onClick: () => void | undefined;
  isDragOverlay?: boolean;
}> = ({ isDragOverlay, icon, label, onClick }) => {
  const token = useAntToken();

  return (
    <div
      onClick={onClick}
      className="flex items-center space-x-2.5 px-2"
      style={{
        borderRadius: token.borderRadius,
        borderColor: token.colorText,
        borderWidth: 0.5,
        borderStyle: 'dotted',
        background: token.colorBgBase,
        height: token.Menu?.itemHeight || token.controlHeight,
        opacity: 0.85,
        transform: 'translateY(-10px)'
      }}>
      {!!icon && icon}
      {label}
    </div>
  );
};
