import { EditableTitle } from '@/components/text';
import React, { useMemo } from 'react';
import { TextPulseLoader } from '@/components/loaders';

export const ChartTitle: React.FC<{
  title: string | undefined | null;
  isReadOnly: boolean;
  editingTitle: boolean;
  setIsEditingTitle: (v: boolean) => void;
  onChangeEditingTitle: (v: string) => void;
  isGeneratingTitle: boolean;
}> = React.memo(
  ({
    title,
    isReadOnly,
    editingTitle,
    setIsEditingTitle,
    onChangeEditingTitle,
    isGeneratingTitle
  }) => {
    const memoizedEditingTitleStyle = useMemo(
      () => ({
        opacity: title ? 1 : 0
      }),
      [title]
    );

    const showLoader = isGeneratingTitle || !title;

    const MemoizedExtraChildren = useMemo(() => {
      return (
        showLoader && (
          <div className="pr-2">
            <TextPulseLoader size={13} showPulseLoader={true} />
          </div>
        )
      );
    }, [showLoader]);

    return (
      <EditableTitle
        style={memoizedEditingTitleStyle}
        className="flex w-full items-center pb-0.5"
        level={3}
        disabled={isReadOnly}
        editing={editingTitle && !isReadOnly}
        onEdit={setIsEditingTitle}
        onChange={onChangeEditingTitle}
        extraChildren={MemoizedExtraChildren}>
        {title || '_title'}
      </EditableTitle>
    );
  }
);
ChartTitle.displayName = 'ChartTitle';
