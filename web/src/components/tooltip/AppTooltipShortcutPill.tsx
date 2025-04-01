import React from 'react';
import isEmpty from 'lodash/isEmpty';
import { useAntToken } from '@/styles/useAntToken';

export const AppTooltipShortcutPill: React.FC<{
  shortcut: string[];
}> = ({ shortcut }) => {
  return (
    <>
      {!isEmpty(shortcut) && (
        <div className="flex space-x-0.5">
          {shortcut.map((s, i) => (
            <TooltipShortcut key={i} shortcut={s} />
          ))}
        </div>
      )}
    </>
  );
};

const TooltipShortcut: React.FC<{ shortcut: string }> = ({ shortcut }) => {
  const { fontSizeLG, colorBorder, borderRadius } = useAntToken();

  return (
    <div
      className="relative flex justify-center"
      style={{
        lineHeight: 1,
        fontSize: 12,
        borderRadius: borderRadius,
        border: `0.5px solid ${colorBorder}`,
        height: fontSizeLG,
        width: fontSizeLG + 2,
        boxShadow: '0 1px 0px 0 rgb(0 0 0 / 0.05)'
      }}>
      <span
        style={{
          transform: 'translateY(0%)'
        }}>
        {shortcut}
      </span>
    </div>
  );
};
