import { useAntToken } from '@/styles/useAntToken';
import React from 'react';

export const PulseLoader: React.FC<{
  size?: number;
  color?: string;
  style?: React.CSSProperties;
}> = ({ style, color, size = 8 }) => {
  const token = useAntToken();

  return (
    <span className="flex flex-col items-center justify-center space-y-4">
      <span
        className="relative flex"
        style={{
          ...style,
          width: `${size}px`,
          height: `${size}px`
        }}>
        <span
          className="absolute inline-flex h-full w-full animate-ping rounded-full opacity-75"
          style={{
            backgroundColor: color || token.colorPrimary
          }}
        />
        <span
          className="relative inline-flex h-full w-full rounded-full"
          style={{
            backgroundColor: color || token.colorPrimary
          }}
        />
      </span>
    </span>
  );
};

export const TextPulseLoader: React.FC<{
  showPulseLoader?: boolean;
  size?: number;
}> = ({ size = 8, showPulseLoader = true }) => {
  const token = useAntToken();

  return (
    <>
      {showPulseLoader && (
        <span
          style={{
            opacity: 0.6,
            display: 'inline-block',
            width: size,
            height: size,
            backgroundColor: token.colorText,
            borderRadius: '100%'
          }}>
          {/* <PulseLoader size={size} color={isDarkMode ? token.colorText : token.colorText} /> */}
        </span>
      )}
    </>
  );
};

export default PulseLoader;
