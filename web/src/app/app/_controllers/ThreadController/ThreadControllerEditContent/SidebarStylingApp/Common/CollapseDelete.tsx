import React, { useMemo, useState } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { createStyles } from 'antd-style';
import { AppMaterialIcons } from '@/components/icons';
import { useMemoizedFn } from 'ahooks';
import { Text } from '@/components/text';
import { SyntheticListenerMap } from '@dnd-kit/core/dist/hooks/utilities';
import { DraggableAttributes } from '@dnd-kit/core';
import { Button } from 'antd';

const ANIMATION_DURATION = 0.145;

interface DraggingProps {
  isDragging?: boolean;
  style?: React.CSSProperties;
  listeners?: SyntheticListenerMap;
  attributes?: DraggableAttributes;
}

const dropdownAnimationConfig = {
  initial: { height: 0, borderTopWidth: 0, opacity: 0 },
  animate: { height: 'auto', borderTopWidth: 0.5, opacity: 1 },
  exit: { height: 0, borderTopWidth: 0, opacity: 0 },
  transition: {
    duration: ANIMATION_DURATION,
    borderTopWidth: { duration: ANIMATION_DURATION / 4 }
  }
};

export const CollapseDelete = React.forwardRef<
  HTMLDivElement,
  {
    children: React.ReactNode;
    title: React.ReactNode | string;
    onDelete?: () => void;
    initialOpen?: boolean;
    draggingProps?: DraggingProps;
  }
>(({ children, title, onDelete, initialOpen = false, draggingProps }, ref) => {
  const { styles, cx } = useStyles();
  const [open, setOpen] = useState(initialOpen);

  const onToggleDropdown = useMemoizedFn(() => {
    setOpen((prev) => !prev);
  });

  return (
    <div className={cx(styles.container, 'flex w-full flex-col')}>
      <CollapseDeleteHeader
        ref={ref}
        title={title}
        onToggleDropdown={onToggleDropdown}
        onClickDelete={onDelete}
        open={open}
        draggingProps={draggingProps}
      />

      <AnimatePresence initial={false}>
        {open && (
          <motion.div
            className={cx(styles.dropdownContent, 'w-full overflow-hidden rounded-b')}
            {...dropdownAnimationConfig}>
            {children}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
});

CollapseDelete.displayName = 'CollapseDelete';

const CollapseDeleteHeader = React.memo(
  React.forwardRef<
    HTMLDivElement,
    {
      title: React.ReactNode | string;
      onToggleDropdown: () => void;
      onClickDelete?: () => void;
      open: boolean;
      draggingProps?: DraggingProps;
    }
  >(({ title, onToggleDropdown, onClickDelete, open, draggingProps }, ref) => {
    const { styles, cx } = useStyles();
    const hasDraggingProps = !!draggingProps;
    const { isDragging, listeners, attributes, style } = draggingProps || {};

    return (
      <div
        ref={ref}
        onClick={onToggleDropdown}
        style={{ ...style }}
        className={cx(
          styles.titleContainer,
          'group flex cursor-pointer select-none items-center justify-between space-x-1',
          isDragging && '!cursor-grabbing shadow-lg'
        )}>
        <div
          {...listeners}
          {...attributes}
          className={cx(
            'flex h-full w-full items-center justify-start overflow-hidden pl-2.5',
            hasDraggingProps && 'cursor-grab'
          )}>
          <TitleComponent title={title} />
        </div>

        <DropdownIcon
          open={open}
          onToggleDropdown={onToggleDropdown}
          onClickDelete={onClickDelete}
          isDragging={isDragging}
        />
      </div>
    );
  })
);
CollapseDeleteHeader.displayName = 'CollapseDeleteHeader';

const TitleComponent: React.FC<{
  title: React.ReactNode | string;
}> = ({ title }) => {
  if (typeof title === 'string') {
    return (
      <Text type="default" className="truncate">
        {title}
      </Text>
    );
  }

  return title;
};

const DropdownIcon: React.FC<{
  open: boolean;
  onToggleDropdown: () => void;
  onClickDelete?: () => void;
  isDragging?: boolean;
}> = React.memo(({ open, onToggleDropdown, onClickDelete, isDragging }) => {
  const { styles, cx } = useStyles();

  const memoizedAnimation = useMemo(() => {
    return {
      initial: { rotate: 0 },
      animate: { rotate: open ? 90 : 0 },
      transition: { duration: ANIMATION_DURATION }
    };
  }, [open]);

  const onClickToggleDropdown = useMemoizedFn((e: React.MouseEvent<HTMLDivElement>) => {
    e.stopPropagation();
    onToggleDropdown();
  });

  const onClickDeletePreflight = useMemoizedFn((e: React.MouseEvent<HTMLDivElement>) => {
    e.stopPropagation();
    onClickDelete?.();
  });

  const onClickContainer = useMemoizedFn((e: React.MouseEvent<HTMLDivElement>) => {
    e.stopPropagation();
  });

  return (
    <div
      className="relative flex h-full cursor-pointer items-center space-x-0.5 pr-1"
      onClick={onClickContainer}>
      {onClickDelete && (
        <Button
          size="small"
          className={cx(
            'flex items-center justify-center',
            'opacity-0 duration-200',
            open ? 'opacity-100' : '',
            'group-hover:flex group-hover:opacity-90',
            'hover:!text-black hover:opacity-100',
            isDragging && '!hidden'
          )}
          type="text"
          icon={<AppMaterialIcons icon="delete" />}
          onClick={onClickDeletePreflight}
        />
      )}

      <Button
        size="small"
        className="flex"
        type="text"
        icon={
          <motion.div
            className={cx(styles.icon, 'flex items-center justify-center', isDragging && '!hidden')}
            {...memoizedAnimation}
            onClick={onClickToggleDropdown}>
            <AppMaterialIcons icon="chevron_right" className={styles.icon} />
          </motion.div>
        }></Button>
    </div>
  );
});
DropdownIcon.displayName = 'DropdownIcon';

const useStyles = createStyles(({ css, token }) => ({
  container: css`
    background: ${token.colorBgContainer};
    border-radius: ${token.borderRadius}px;
    border: 0.5px solid ${token.colorBorder};
  `,
  titleContainer: css`
    height: ${token.controlHeight + 4}px;
    min-height: ${token.controlHeight + 4}px;
    max-height: ${token.controlHeight + 4}px;
  `,
  icon: css`
    color: ${token.colorIcon};
  `,
  dropdownContent: css`
    border-top: 0.5px solid ${token.colorBorder};
  `
}));
