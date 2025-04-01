import { NextRequest, NextResponse } from 'next/server';

export const nextPathnameMiddleware = (request: NextRequest, response: NextResponse) => {
  response.headers.set('x-next-pathname', request.nextUrl.pathname);
  response.cookies.set('x-next-pathname', request.nextUrl.pathname, {
    secure: true,
    httpOnly: true,
    sameSite: 'lax'
  });
};
