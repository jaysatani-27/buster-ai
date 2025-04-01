import { useMemo } from 'react';
import { DropZone, DropZoneInternal } from './interfaces';
import { useMemoizedFn, useUpdateEffect } from 'ahooks';

export const useDropzonesExternal = ({
  onChange
}: {
  onChange?: (dropZones: DropZone[]) => void;
}) => {
  const onDropzonesChange = useMemoizedFn((dropZones: DropZoneInternal[]) => {
    if (onChange) {
      onChange(
        dropZones.map((zone) => ({
          ...zone,
          items: zone.items.map(({ originalId }) => originalId)
        }))
      );
    }
  });

  return { onDropzonesChange };
};
