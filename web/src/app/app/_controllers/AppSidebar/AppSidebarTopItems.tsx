import React from 'react';
import { Button, ConfigProvider } from 'antd';
import Link from 'next/link';
import { BusterRoutes, BusterRoutesWithArgsRoute } from '@/routes';
import { AppMaterialIcons } from '@/components/icons';
import { BusterLogoNew } from '@/assets/svg/BusterLogoNew';
import { AppTooltip } from '@/components';
import { useHotkeys } from 'react-hotkeys-hook';
import { useAntToken } from '@/styles/useAntToken';
import { ThemeConfig } from 'antd/lib';

const THEME_CONFIG_1: ThemeConfig = {
  components: {
    Button: {
      controlHeight: 28,
      borderRadius: 10
    }
  }
};

const THEME_CONFIG_2: ThemeConfig = {
  components: {
    Button: {
      defaultShadow: `0px 2px 6px 0px rgba(0, 0, 0, 0.05);`
    }
  }
};

export const AppSidebarTopItems: React.FC<{
  onOpenSettings: () => void;
  onGoToHomePage: () => void;
  onOpenThreadsModal: () => void;
  createPageLink: (params: BusterRoutesWithArgsRoute) => string;
  className?: string;
  threadModalOpen: boolean;
  style?: React.CSSProperties;
  isUserRegistered: boolean;
}> = React.memo(
  ({
    style,
    onOpenSettings,
    onGoToHomePage,
    createPageLink,
    onOpenThreadsModal,
    className = '',
    isUserRegistered
  }) => {
    useHotkeys('g+h', onGoToHomePage);
    useHotkeys('g+s', onOpenSettings);
    useHotkeys('m', onOpenThreadsModal);

    return (
      <div className={`flex items-center justify-between ${className}`} style={style}>
        <LogoLink isUserRegistered={isUserRegistered} createPageLink={createPageLink} />

        <AppSidebarButton
          createPageLink={createPageLink}
          isUserRegistered={isUserRegistered}
          onOpenThreadsModal={onOpenThreadsModal}
        />
      </div>
    );
  }
);

AppSidebarTopItems.displayName = 'AppSidebarTopItems';

const AppSidebarButton = React.memo(
  ({
    createPageLink,
    isUserRegistered,
    onOpenThreadsModal
  }: {
    createPageLink: (params: BusterRoutesWithArgsRoute) => string;
    isUserRegistered: boolean;
    onOpenThreadsModal: () => void;
  }) => {
    return (
      <ConfigProvider theme={THEME_CONFIG_1}>
        <div className="flex items-center space-x-2.5">
          <Link
            href={createPageLink({
              route: BusterRoutes.SETTINGS_GENERAL
            })}>
            <AppTooltip title="Settings" shortcuts={['G', 'S']}>
              <Button
                type={'text'}
                disabled={!isUserRegistered}
                icon={<AppMaterialIcons size={18} icon="settings" />}
              />
            </AppTooltip>
          </Link>
          <ConfigProvider theme={THEME_CONFIG_2}>
            <AppTooltip title="New metric" shortcuts={['M']}>
              <Button
                data-cy="new-metric-button"
                disabled={!isUserRegistered}
                type={'default'}
                onClick={onOpenThreadsModal}
                icon={<AppMaterialIcons icon="edit_square" size={16} />}
              />
            </AppTooltip>
          </ConfigProvider>
        </div>
      </ConfigProvider>
    );
  }
);
AppSidebarButton.displayName = 'AppSidebarButton';

const LogoLink = React.memo(
  ({
    isUserRegistered,
    createPageLink
  }: {
    isUserRegistered: boolean;
    createPageLink: (params: BusterRoutesWithArgsRoute) => string;
  }) => {
    const token = useAntToken();

    return (
      <Link
        href={
          isUserRegistered
            ? createPageLink({
                route: BusterRoutes.APP_THREAD
              })
            : '/'
        }>
        <AppTooltip title={'Home'} shortcuts={['G', 'H']}>
          <BusterLogoNew
            style={{ color: isUserRegistered ? undefined : token.colorTextDisabled }}
          />
        </AppTooltip>
      </Link>
    );
  }
);
LogoLink.displayName = 'LogoLink';
