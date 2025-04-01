import React, { PropsWithChildren } from 'react';
import { WelcomeToBuster } from './login/_components/WelcomeSidebar';
import { Metadata } from 'next';
import { LoginConfigProvider } from './login/_components/LoginConfigProvider';

export const metadata: Metadata = {
  title: 'Buster Login'
};

const LoginLayout: React.FC<PropsWithChildren<{}>> = ({ children }) => {
  return (
    <LoginConfigProvider>
      <section className="h-[100vh]">
        <div className="flex h-[100vh] items-center">
          <div className="mx-auto flex min-h-[100vh] w-full">
            <div className="hidden w-1/2 w-full bg-gray-50 dark:bg-gray-900 md:flex">
              <WelcomeToBuster hasUser={true} />
            </div>
            <div className="w-1/2 w-full bg-white dark:bg-black">{children}</div>
          </div>
        </div>
      </section>
    </LoginConfigProvider>
  );
};

export default LoginLayout;
