import React from 'react';

export default function Layout({
  children,
  params: { dashboardId, threadId }
}: Readonly<{
  children: React.ReactNode;
  params: { dashboardId: string; threadId: string };
}>) {
  return <>{children}</>;
}
