'use client';

import { BusterRoutesWithArgsRoute, createBusterRoute } from '@/routes/busterRoutes';
import { pathNameToRoute } from '@/routes/helpers';
import { useMemoizedFn, usePrevious } from 'ahooks';
import { useRouter, usePathname, useSelectedLayoutSegment, useParams } from 'next/navigation';
import React, { PropsWithChildren } from 'react';
import {
  createContext,
  ContextSelector,
  useContextSelector
} from '@fluentui/react-context-selector';

export const useAppLayout = () => {
  const { push } = useRouter();
  const pathname = usePathname();
  const params = useParams();
  const currentSegment = useSelectedLayoutSegment();
  const currentRoute = pathNameToRoute(pathname, params);
  const previousRoute = usePrevious(currentRoute);
  const previousPath = usePrevious(pathname);
  const [openThreadsModal, setOpenThreadsModal] = React.useState(false);
  const [openInviteModal, setOpenInviteModal] = React.useState(false);
  const [openSupportModal, setOpenSupportModal] = React.useState(false);

  const onToggleThreadsModal = useMemoizedFn((v?: boolean) => {
    setOpenThreadsModal(v ?? !openThreadsModal);
  });

  const onToggleInviteModal = useMemoizedFn((v?: boolean) => {
    setOpenInviteModal(v ?? !openInviteModal);
  });

  const onToggleSupportModal = useMemoizedFn((v?: boolean) => {
    setOpenSupportModal(v ?? !openSupportModal);
  });

  const createPageLink = useMemoizedFn((params: BusterRoutesWithArgsRoute) => {
    return createBusterRoute(params);
  });

  const onChangePage = useMemoizedFn((params: BusterRoutesWithArgsRoute) => {
    push(createBusterRoute(params));
  });

  return {
    onToggleThreadsModal,
    createPageLink,
    currentRoute,
    currentSegment,
    openThreadsModal,
    onToggleInviteModal,
    openInviteModal,
    onChangePage,
    pathname,
    previousPath,
    openSupportModal,
    previousRoute,
    onToggleSupportModal
  };
};

const AppLayoutContext = createContext<ReturnType<typeof useAppLayout>>(
  {} as ReturnType<typeof useAppLayout>
);

export const AppLayoutProvider = React.memo<PropsWithChildren>(({ children }) => {
  const value = useAppLayout();

  return <AppLayoutContext.Provider value={value}>{children}</AppLayoutContext.Provider>;
});
AppLayoutProvider.displayName = 'AppLayoutProvider';

export const useAppLayoutContextSelector = <T,>(
  selector: ContextSelector<ReturnType<typeof useAppLayout>, T>
) => useContextSelector(AppLayoutContext, selector);
