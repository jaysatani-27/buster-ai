import { AppTooltip, AppMaterialIcons } from '@/components';
import { IBusterThread, useBusterThreadsContextSelector } from '@/context/Threads';
import { useMemoizedFn } from 'ahooks';
import { Button } from 'antd';
import { createStyles } from 'antd-style';
import React from 'react';

export const UndoRedoContainer: React.FC<{
  threadId: IBusterThread['id'];
}> = React.memo(({ threadId }) => {
  const { cx, styles } = useStyles();
  const canRedo = useBusterThreadsContextSelector((x) => x.canRedo(threadId));
  const canUndo = useBusterThreadsContextSelector((x) => x.canUndo(threadId));
  const onUndo = useBusterThreadsContextSelector((x) => x.onUndo);
  const onRedo = useBusterThreadsContextSelector((x) => x.onRedo);

  const disableUndo = !canUndo;
  const disableRedo = !canRedo;

  const onClickUndo = useMemoizedFn(() => {
    if (!disableUndo) onUndo({ threadId });
  });

  const onClickRedo = useMemoizedFn(() => {
    if (!disableRedo) onRedo({ threadId });
  });

  return (
    <div className="flex h-full items-center space-x-2">
      <div className={cx(styles.divider, 'h-[80%] w-0')} />

      <div className="flex w-full items-center justify-center space-x-1 px-1">
        <AppTooltip title={'Undo'}>
          <Button
            icon={<AppMaterialIcons icon="undo" size={18} />}
            type="text"
            disabled={disableUndo}
            onClick={onClickUndo}
          />
        </AppTooltip>
        <AppTooltip title={'Redo'}>
          <Button
            icon={<AppMaterialIcons icon="redo" size={18} />}
            type="text"
            disabled={disableRedo}
            onClick={onClickRedo}
          />
        </AppTooltip>
      </div>
    </div>
  );
});
UndoRedoContainer.displayName = 'UndoRedoContainer';

const useStyles = createStyles(({ token }) => {
  return {
    divider: {
      borderRight: `0.5px solid ${token.colorBorder}`
    }
  };
});
