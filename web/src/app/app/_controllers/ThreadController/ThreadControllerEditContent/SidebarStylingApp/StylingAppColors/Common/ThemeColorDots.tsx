import { createStyles } from 'antd-style';
import React from 'react';

export const ThemeColorDots: React.FC<{ colors: string[]; numberOfColors?: number | 'all' }> = ({
  colors,
  numberOfColors = 'all'
}) => {
  const { cx, styles } = useStyles();

  const numberOfColorsToShow = numberOfColors === 'all' ? colors.length : numberOfColors;

  return (
    <div className="flex shrink-0 items-center gap-0">
      {colors.slice(0, numberOfColorsToShow).map((color, colorIdx) => (
        <div
          key={colorIdx}
          className={cx('rounded-full', colorIdx > 0 && '-ml-0.5', 'ball', styles.ball)}
          style={{ backgroundColor: color }}
        />
      ))}
    </div>
  );
};
ThemeColorDots.displayName = 'ThemeColorDots';

const useStyles = createStyles(({ css, token }) => ({
  ball: css`
    width: 8px;
    height: 8px;
    box-shadow: 0 0 0 0.75px ${token.controlItemBgActive};
  `
}));
