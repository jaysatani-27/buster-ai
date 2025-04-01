import React from 'react';
import { LoginConfigProvider } from '@/app/auth/login/_components/LoginConfigProvider';
import { NewUserController } from './_NewUserController';

export default function NewUserPage() {
  return (
    <LoginConfigProvider>
      <NewUserController />
    </LoginConfigProvider>
  );
}
