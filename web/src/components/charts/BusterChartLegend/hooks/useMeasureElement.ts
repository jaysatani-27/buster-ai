import { useCallback, useEffect, useRef, useState } from 'react';

export const useMeasureElement = (useMeasure?: boolean) => {
  const elementRef = useRef<HTMLDivElement>(null);
  const [dimensions, setDimensions] = useState({ fullWidth: 0, fullHeight: 0 });

  const updateDimensions = useCallback(() => {
    if (elementRef.current) {
      const rect = elementRef.current.getBoundingClientRect();

      setDimensions({
        fullWidth: rect.width,
        fullHeight: rect.height
      });
    }
  }, []);

  useEffect(() => {
    if (!elementRef.current || !useMeasure) return;

    const resizeObserver = new ResizeObserver(updateDimensions);
    resizeObserver.observe(elementRef.current);

    return () => {
      resizeObserver.disconnect();
    };
  }, [updateDimensions, useMeasure]);

  return {
    elementRef,
    ...dimensions
  };
};
