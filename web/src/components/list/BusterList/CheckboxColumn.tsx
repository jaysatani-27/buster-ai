import { useMemoizedFn } from 'ahooks';
import React from 'react';
import { MemoizedCheckbox } from './MemoizedCheckbox';
import { createStyles } from 'antd-style';
import { WIDTH_OF_CHECKBOX_COLUMN } from './config';

export const CheckboxColumn: React.FC<{
  checkStatus: 'checked' | 'unchecked' | 'indeterminate' | undefined;
  onChange: (v: boolean) => void;
  className?: string;
}> = React.memo(({ checkStatus, onChange, className = '' }) => {
  const { styles, cx } = useStyles();
  const showBox = checkStatus === 'checked'; //|| checkStatus === 'indeterminate';

  const onClickStopPropagation = useMemoizedFn((e: React.MouseEvent<HTMLDivElement>) => {
    e.stopPropagation();
  });

  return (
    <div
      onClick={onClickStopPropagation}
      className={cx(
        styles.checkboxColumn,
        'flex items-center justify-center opacity-0',
        className,
        'group-hover:opacity-100',
        showBox ? 'opacity-100' : ''
      )}>
      <MemoizedCheckbox
        checked={checkStatus === 'checked'}
        indeterminate={checkStatus === 'indeterminate'}
        onChange={onChange}
      />
    </div>
  );
});
CheckboxColumn.displayName = 'CheckboxColumn';

export const useStyles = createStyles(({ css, token }) => ({
  checkboxColumn: css`
    padding-left: 4px;
    padding-right: 0px;
    width: ${WIDTH_OF_CHECKBOX_COLUMN}px;
    min-width: ${WIDTH_OF_CHECKBOX_COLUMN}px;
    display: flex;
    align-items: center;
    justify-content: center;
  `
}));
