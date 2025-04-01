import { useState } from 'react';
import { SelectAxisContainerId } from '../config';
import { useMemoizedFn } from 'ahooks';
import { DropZoneInternal } from './interfaces';
import { useSelectAxisContextSelector } from '../useSelectAxisContext';
import { ChartType, IColumnLabelFormat } from '@/components/charts';
import { Active } from '@dnd-kit/core';
import { isNumericColumnStyle, isNumericColumnType } from '@/utils';

interface ZoneError {
  error: boolean;
  reason: string | null;
  zoneId: SelectAxisContainerId;
}

export const useErrorZones = () => {
  const selectedChartType = useSelectAxisContextSelector((x) => x.selectedChartType);
  const columnLabelFormats = useSelectAxisContextSelector((x) => x.columnLabelFormats);
  const [errorZone, setErrorZone] = useState<ZoneError | null>(null);

  const onDragOverCheckErrorZone = useMemoizedFn(
    (
      targetZone: DropZoneInternal | null,
      sourceZoneId: SelectAxisContainerId | null,
      activeItem: Active
    ) => {
      if (targetZone && sourceZoneId) {
        if (sourceZoneId !== targetZone.id) {
          const originalItemId = activeItem.data?.current?.item.originalId;
          const columnLabelFormat = columnLabelFormats[originalItemId];
          const zoneError = checkForError(
            targetZone,
            originalItemId,
            columnLabelFormat,
            selectedChartType
          );
          if (zoneError) {
            setErrorZone(zoneError);
          } else {
            setErrorZone(null);
          }

          return;
        }

        setErrorZone(null); // Clear error state when dragging within the same zone
      }
    }
  );

  return { errorZone, setErrorZone, onDragOverCheckErrorZone };
};

const checkDuplicates = (targetZone: DropZoneInternal, activeItemOriginalId: string): boolean => {
  return targetZone.items.some((item) => item.originalId === activeItemOriginalId);
};

const zoneErrorRecord: Record<
  SelectAxisContainerId,
  (
    targetZone: DropZoneInternal,
    columnLabelFormat: Required<IColumnLabelFormat>,
    selectedChartType: ChartType
  ) => {
    error: boolean;
    reason: string;
  } | null
> = {
  [SelectAxisContainerId.Available]: () => null,
  [SelectAxisContainerId.Metric]: () => null,
  [SelectAxisContainerId.XAxis]: (targetZone, columnLabelFormat, selectedChartType) => {
    return null;
  },
  [SelectAxisContainerId.YAxis]: (targetZone, columnLabelFormat) => {
    const isNumericType = isNumericColumnType(columnLabelFormat.columnType);
    if (!isNumericType) {
      return {
        error: true,
        reason: 'Y-axis must be numeric column type',
        zoneId: targetZone.id
      };
    }

    const isNumericStyle = isNumericColumnStyle(columnLabelFormat.style);
    if (!isNumericStyle) {
      return {
        error: true,
        reason: 'Y-axis must be a number style (number, currency, percentage)',
        zoneId: targetZone.id
      };
    }

    return null;
  },
  [SelectAxisContainerId.Y2Axis]: (targetZone, columnLabelFormat) => {
    const isNumericType = isNumericColumnType(columnLabelFormat.columnType);
    if (!isNumericType) {
      return {
        error: true,
        reason: 'Right Y-axis must be numeric column type',
        zoneId: targetZone.id
      };
    }

    const isNumericStyle = isNumericColumnStyle(columnLabelFormat.style);
    if (!isNumericStyle) {
      return {
        error: true,
        reason: 'Right Y-axis must be a number style (number, currency, percentage)',
        zoneId: targetZone.id
      };
    }

    return null;
  },
  [SelectAxisContainerId.CategoryAxis]: () => null,
  [SelectAxisContainerId.SizeAxis]: (targetZone) => {
    if (targetZone.items.length >= 1) {
      return {
        error: true,
        reason: 'Cannot add more than one size column'
      };
    }
    return null;
  },
  [SelectAxisContainerId.Tooltip]: () => null
};

const checkForError = (
  targetZone: DropZoneInternal,
  activeItemOriginalId: string,
  columnLabelFormat: Required<IColumnLabelFormat>,
  selectedChartType: ChartType
): ZoneError | null => {
  const hasDuplicate = checkDuplicates(targetZone, activeItemOriginalId);

  if (hasDuplicate) {
    return {
      error: true,
      reason: 'Cannot add duplicate column',
      zoneId: targetZone.id
    };
  }

  const targetZoneId = targetZone.id;
  const zoneError = zoneErrorRecord[targetZoneId](targetZone, columnLabelFormat, selectedChartType);

  if (!zoneError) return null;

  return {
    ...zoneError,
    zoneId: targetZone.id
  };
};
