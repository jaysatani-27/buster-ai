import { ConfigProvider } from 'antd';
import { ThemeConfig } from 'antd/lib';
import React from 'react';

const theme: ThemeConfig = {
  token: {
    controlHeight: 28
  },
  components: {
    Button: {
      colorPrimary: 'black',
      defaultHoverBg: 'rgba(0,0,0,0.04)',
      defaultActiveBg: 'black',
      colorPrimaryBg: 'black',
      colorPrimaryBgHover: 'transparent',
      colorPrimaryHover: 'rgba(0,0,0,0.88)',
      colorPrimaryActive: 'rgba(0,0,0,0.95)'
    }
  }
};

export const LoginConfigProvider: React.FC<{
  children: React.ReactNode;
}> = React.memo(({ children }) => {
  return <ConfigProvider theme={theme}>{children}</ConfigProvider>;
});

LoginConfigProvider.displayName = 'LoginConfigProvider';
