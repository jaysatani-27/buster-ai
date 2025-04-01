'use client';

import { Result } from 'antd';

export default function AuthCodeErrorPage(p: { params: {}; searchParams: {} }) {
  return (
    <div className="flex h-[100vh] w-full items-center justify-center p-4">
      <Result status="error" title="Auth Code Error" subTitle="Please contact support." />
    </div>
  );
}
