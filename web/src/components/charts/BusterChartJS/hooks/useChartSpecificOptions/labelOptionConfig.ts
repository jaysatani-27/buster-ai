import { busterAppStyleConfig } from '@/styles/busterAntDStyleConfig';

const token = busterAppStyleConfig.token!;

export const defaultLabelOptionConfig = {
  backgroundColor: token.colorBgContainerDisabled,
  borderWidth: 0.5,
  borderColor: token.colorBorder,
  borderRadius: token.borderRadius,
  padding: {
    top: 3,
    bottom: 3,
    left: 6,
    right: 6
  },
  color: token.colorTextSecondary,
  font: {
    size: 10,
    weight: 'normal' as 'normal'
  }
};
