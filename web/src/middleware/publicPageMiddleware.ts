import { BusterAuthRoutes } from '@/routes/busterRoutes/busterAuthRoutes';
import { BusterRoutes, createPathnameToBusterRoute } from '@/routes/busterRoutes/busterRoutes';
import { NextRequest } from 'next/server';

const assetCheckPages: BusterRoutes[] = [
  BusterRoutes.APP_THREAD_ID,
  BusterRoutes.APP_DASHBOARD_ID,
  BusterRoutes.APP_DASHBOARD_THREADS_ID
];

const publicPages: BusterRoutes[] = [
  BusterRoutes.APP_THREAD_ID,
  BusterRoutes.APP_DASHBOARD_ID,
  BusterRoutes.APP_DASHBOARD_THREADS_ID,
  ...Object.values(BusterAuthRoutes)
];

export const isPublicPage = (request: NextRequest): boolean => {
  const route = request.nextUrl.pathname;
  const matchedRoute = createPathnameToBusterRoute(route);
  return publicPages.some((page) => page === matchedRoute);
};

export const assetPermissionCheck = (request: NextRequest): boolean => {
  const route = request.nextUrl.pathname;
  const matchedRoute = createPathnameToBusterRoute(route);
  return assetCheckPages.includes(matchedRoute);
};
