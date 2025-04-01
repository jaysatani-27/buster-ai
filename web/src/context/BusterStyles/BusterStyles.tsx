'use client';

import 'react-material-symbols/rounded';
import React, { PropsWithChildren } from 'react';
import { usePreventBackwardNavigation } from '@/hooks';
import { App, ConfigProvider, theme, ThemeConfig } from 'antd';
import {
  busterAppStyleConfig,
  busterAppStyleConfigDark,
  useBusterAppComponentConfig
} from '@/styles/busterAntDStyleConfig';
import { ThemeProvider } from 'antd-style';
import { ThemeProvider as NextThemeProvider, useTheme as useNextTheme } from 'next-themes';
import StyleRegistry from './StyleRegistry';
import { AntdRegistry } from '@ant-design/nextjs-registry';
import { configResponsive } from 'ahooks';
import {
  useContextSelector,
  createContext,
  ContextSelector
} from '@fluentui/react-context-selector';
import tailwind from '../../../tailwind.config';

const screens = tailwind.theme.screens;

const { defaultAlgorithm, darkAlgorithm } = theme;

configResponsive({
  xs: parseInt(screens.xs),
  sm: parseInt(screens.sm),
  md: parseInt(screens.md),
  lg: parseInt(screens.lg),
  xl: parseInt(screens.xl),
  '2xl': parseInt(screens['2xl'])
});

export const ENABLE_DARK_MODE = false;

export const BaseBusterStyleProvider: React.FC<PropsWithChildren<{}>> = React.memo(
  ({ children }) => {
    usePreventBackwardNavigation();

    const nextThemeMode = useNextTheme();
    const isDarkMode =
      ENABLE_DARK_MODE &&
      (nextThemeMode.theme === 'dark' || nextThemeMode.resolvedTheme === 'dark');
    const selectedTheme = !isDarkMode ? busterAppStyleConfig : busterAppStyleConfigDark;
    const selectedAlgorithm = isDarkMode ? darkAlgorithm : defaultAlgorithm;
    const token = selectedTheme.token!;

    const busterAppComponentConfig = useBusterAppComponentConfig();

    const cssVariables = {
      '--focus-border': '#E5E5E5',
      '--separator-border': token.colorBorder
    };

    return (
      <StyleRegistry>
        <BusterStyles.Provider value={{ isDarkMode, theme: selectedTheme }}>
          <ThemeProvider
            themeMode={isDarkMode ? 'dark' : 'light'}
            appearance={isDarkMode ? 'dark' : 'light'}
            prefixCls="busterv2"
            theme={{
              algorithm: selectedAlgorithm,
              ...selectedTheme
            }}>
            <ConfigProvider
              popupMatchSelectWidth={false}
              virtual={true}
              locale={{
                locale: 'en'
              }}
              {...busterAppComponentConfig}>
              <App
                message={{
                  maxCount: 4,
                  duration: 2.5
                }}
                className="min-h-[100vh] w-full text-base"
                style={{
                  ...(cssVariables as any)
                }}>
                {children}
                <div id="modal-root"></div>
              </App>
            </ConfigProvider>
          </ThemeProvider>
        </BusterStyles.Provider>
      </StyleRegistry>
    );
  }
);
BaseBusterStyleProvider.displayName = 'BusterStyleProvider';

export const BusterStyleProvider: React.FC<PropsWithChildren<{}>> = ({ children }) => {
  return (
    <AntdRegistry>
      <NextThemeProvider
        attribute="class"
        enableSystem={ENABLE_DARK_MODE}
        themes={['light', 'dark']}>
        <BaseBusterStyleProvider>{children}</BaseBusterStyleProvider>
      </NextThemeProvider>
    </AntdRegistry>
  );
};

const BusterStyles = createContext<{ isDarkMode: boolean; theme: ThemeConfig }>({
  isDarkMode: false,
  theme: busterAppStyleConfig
});

export const useBusterStylesContext = <T,>(
  selector: ContextSelector<
    {
      isDarkMode: boolean;
      theme: ThemeConfig;
    },
    T
  >
) => {
  return useContextSelector(BusterStyles, selector);
};
