import React from 'react';
import { IColorTheme } from './interfaces';
import { Text } from '@/components/text';
import { createStyles } from 'antd-style';
import { ThemeColorDots } from './ThemeColorDots';

export const ThemeList: React.FC<{
  themes: {
    selected: boolean;
    name: string;
    colors: string[];
  }[];
  onChangeColorTheme: (theme: IColorTheme) => void;
}> = ({ themes, onChangeColorTheme }) => {
  const { styles, cx } = useStyles();

  return (
    <div className={cx(styles.container, 'flex w-full flex-col space-y-0 overflow-y-auto')}>
      {themes.map((theme) => (
        <ColorOption
          key={theme.name}
          theme={theme}
          selected={theme.selected}
          onChangeColorTheme={onChangeColorTheme}
        />
      ))}
    </div>
  );
};

const ColorOption: React.FC<{
  theme: IColorTheme;
  selected: boolean;
  onChangeColorTheme: (theme: IColorTheme) => void;
}> = React.memo(({ theme, selected, onChangeColorTheme }) => {
  const { styles, cx } = useStyles();
  const { name, colors } = theme;

  return (
    <div
      onClick={() => onChangeColorTheme(theme)}
      className={cx(
        styles.itemContainer,
        selected && 'selected',
        'flex w-full items-center justify-between'
      )}>
      <div className="flex items-center space-x-2">
        <Text>{name}</Text>
      </div>

      <ThemeColorDots colors={colors} />
    </div>
  );
});
ColorOption.displayName = 'ColorOption';

const useStyles = createStyles(({ css, token }) => ({
  container: css`
    padding: 4px;
    border: 0.5px solid ${token.colorBorder};
    border-radius: 4px;
    background-color: ${token.controlItemBgActive};
  `,
  itemContainer: css`
    padding: 6px 12px;
    border-radius: 4px;
    cursor: pointer;

    &.selected {
      box-shadow: inset 0 0 0 0.5px ${token.colorBorder};
      background-color: ${token.colorBgContainer};

      .ball {
        box-shadow: 0 0 0 0.75px ${token.colorBgContainer};
      }

      &:hover {
        background-color: ${token.colorBgContainer};
      }
    }

    &:hover {
      background-color: ${token.controlItemBgHover};
      box-shadow: inset 0 0 0 0.5px ${token.colorBorder};

      .ball {
        box-shadow: 0 0 0 0.75px ${token.controlItemBgHover};
      }
    }
  `
}));
