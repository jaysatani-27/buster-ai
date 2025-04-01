import { Menu } from 'antd';
import React from 'react';

export const SignOutButton: React.FC<{
  signOut: () => void;
}> = ({ signOut }) => {
  return (
    <Menu
      items={[
        {
          key: 'signout',
          label: 'Sign out',
          onClick: () => {
            signOut();
          }
        }
      ]}
    />
  );
};
