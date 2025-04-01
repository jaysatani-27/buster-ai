import React, { useMemo } from 'react';
import { Title } from '@/components/text';

const descriptionEllipsis = {
  tooltip: true
};

export const ChartSubTitle: React.FC<{
  timeFrame: string | undefined | null;
  description: string | undefined | null;
}> = React.memo(({ timeFrame, description }) => {
  const memoizedTimeFrameStyle = useMemo(() => {
    return {
      opacity: !timeFrame && !description ? 0 : 1
    };
  }, [timeFrame, description]);

  const memoizedDescriptionStyle = useMemo(() => {
    return {
      opacity: description ? 1 : 0
    };
  }, [description]);

  return (
    <div
      className="flex w-full items-center space-x-1 overflow-hidden text-nowrap"
      style={memoizedTimeFrameStyle}>
      <Title
        className={`text-nowrap ${timeFrame ? 'inline-block' : 'hidden'}`}
        level={4}
        type="secondary"
        ellipsis={false}>
        {timeFrame || '_time_frame'}
      </Title>
      {!!description && !!timeFrame && <Title level={4}>{` • `}</Title>}
      <Title
        level={4}
        style={memoizedDescriptionStyle}
        type="secondary"
        ellipsis={descriptionEllipsis}>
        {description || '_description'}
      </Title>
    </div>
  );
});
ChartSubTitle.displayName = 'ChartSubTitle';
