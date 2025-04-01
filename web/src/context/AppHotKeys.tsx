import React, { PropsWithChildren, useRef } from 'react';
import { HotkeysProvider } from 'react-hotkeys-hook';

export const AppHotKeysProvider: React.FC<PropsWithChildren> = ({ children }) => {
  return <HotkeysProvider initiallyActiveScopes={['app']}>{children}</HotkeysProvider>;
};
