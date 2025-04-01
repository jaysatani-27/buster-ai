import React from 'react';
import { useBusterSupabaseAuthMethods } from '@/hooks/useBusterSupabaseAuthMethods';
import { ResetEmailForm } from './_ResetEmailForm';

export default function ResetPassword(p: { searchParams: { email: string } }) {
  const queryEmail = p.searchParams.email;
  const { resetPasswordEmailSend } = useBusterSupabaseAuthMethods();

  return <ResetEmailForm queryEmail={queryEmail} resetPasswordEmailSend={resetPasswordEmailSend} />;
}
