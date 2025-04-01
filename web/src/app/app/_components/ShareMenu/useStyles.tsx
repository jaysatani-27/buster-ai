import { createStyles } from 'antd-style';

export const useStyles = createStyles(({ css, token }) => ({
  hoverListItem: css`
    &:hover {
      background: ${token.controlItemBgHover};
    }
  `
}));
