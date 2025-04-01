'use client';

import { registerTheme } from 'echarts';
import { useBusterStylesContext } from '@/context/BusterStyles';
import { useAsyncEffect } from 'ahooks';
import { busterLightTheme } from './buster_light_theme';

export const busterLightThemeName = 'buster_light';
export const busterDarkThemeName = 'buster_light';

registerTheme(busterDarkThemeName, busterLightTheme.theme);
registerTheme(busterLightThemeName, busterLightTheme.theme);

export const useEchartThemes = () => {
  const isDarkMode = useBusterStylesContext((s) => s.isDarkMode);
  const theme = isDarkMode ? busterDarkThemeName : busterLightThemeName;

  useAsyncEffect(async () => {
    if (isDarkMode) {
      // echarts.registerTheme(busterDarkThemeName, busterLightTheme.theme);
    } else {
      //  echarts.registerTheme(busterLightThemeName, busterLightTheme.theme);
    }
  }, []);

  return theme;
};
