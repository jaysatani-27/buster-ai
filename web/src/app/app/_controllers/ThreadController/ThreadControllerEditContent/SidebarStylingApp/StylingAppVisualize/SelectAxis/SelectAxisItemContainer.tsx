import React from 'react';
import { SelectAxisItemLabel } from './SelectAxisItemLabel';
import { useSelectAxisContextSelector } from './useSelectAxisContext';
import { useMemoizedFn } from 'ahooks';
import { chartTypeToAxis, SelectAxisContainerId, zoneIdToAxis } from './config';
import { SelectAxisDropdownContent } from './SelectAxisColumnContent';
import { useBusterThreadsContextSelector } from '@/context/Threads';
import { ChartEncodes, IColumnLabelFormat } from '@/components/charts';
import { CollapseDelete } from '../../Common/CollapseDelete';
import { SyntheticListenerMap } from '@dnd-kit/core/dist/hooks/utilities';
import { DraggableAttributes } from '@dnd-kit/core';

interface SelectAxisItemContainerProps {
  id: string;
  zoneId: SelectAxisContainerId;
  isPlaceholder?: boolean;
  //DRAGGING PROPERTIES
  isDragging?: boolean;
  style?: React.CSSProperties;
  listeners?: SyntheticListenerMap;
  attributes?: DraggableAttributes;
}

export const SelectAxisItemContainer = React.memo(
  React.forwardRef<HTMLDivElement, SelectAxisItemContainerProps>(
    ({ id, zoneId, isPlaceholder, ...draggingProps }, ref) => {
      const columnLabelFormat: undefined | IColumnLabelFormat = useSelectAxisContextSelector(
        (x) => x.columnLabelFormats[id]
      );
      const selectedAxis = useSelectAxisContextSelector((x) => x.selectedAxis);
      const selectedChartType = useSelectAxisContextSelector((x) => x.selectedChartType);
      const onUpdateMessageChartConfig = useBusterThreadsContextSelector(
        (x) => x.onUpdateMessageChartConfig
      );

      const onDelete = useMemoizedFn(() => {
        if (selectedAxis && selectedChartType) {
          const axis = zoneIdToAxis[zoneId] as keyof ChartEncodes;
          if (!axis || !selectedAxis[axis]) return;

          const newSelectedAxis: ChartEncodes = {
            ...selectedAxis,
            [axis]: (selectedAxis[axis] as string[]).filter((x) => x !== id)
          };

          if (chartTypeToAxis[selectedChartType]) {
            onUpdateMessageChartConfig({
              chartConfig: {
                [chartTypeToAxis[selectedChartType]]: newSelectedAxis
              }
            });
          }
        }
      });

      return (
        <div
          className={`transition-opacity duration-200 ${isPlaceholder ? 'opacity-0' : 'opacity-100'}`}>
          <CollapseDelete
            ref={ref}
            draggingProps={draggingProps}
            title={<SelectAxisItemLabel id={id} columnLabelFormat={columnLabelFormat} />}
            onDelete={onDelete}>
            <DropdownContent id={id} zoneId={zoneId} />
          </CollapseDelete>
        </div>
      );
    }
  )
);
SelectAxisItemContainer.displayName = 'SelectAxisItemContainer';

const DropdownContent: React.FC<{ id: string; zoneId: SelectAxisContainerId }> = ({
  id,
  zoneId
}) => {
  const columnLabelFormat = useSelectAxisContextSelector((x) => x.columnLabelFormats[id]);
  const columnSetting = useSelectAxisContextSelector((x) => x.columnSettings[id]);
  const selectedChartType = useSelectAxisContextSelector((x) => x.selectedChartType);
  const barGroupType = useSelectAxisContextSelector((x) => x.barGroupType);
  const lineGroupType = useSelectAxisContextSelector((x) => x.lineGroupType);
  const selectedAxis = useSelectAxisContextSelector((x) => x.selectedAxis);

  return (
    <SelectAxisDropdownContent
      hideTitle
      id={id}
      zoneId={zoneId}
      columnLabelFormat={columnLabelFormat}
      columnSetting={columnSetting}
      selectedChartType={selectedChartType}
      barGroupType={barGroupType}
      lineGroupType={lineGroupType}
      selectedAxis={selectedAxis}
    />
  );
};
