import { BusterAppRoutes, BusterAppRoutesWithArgs } from './busterAppRoutes';
import { BusterAuthRoutes, BusterAuthRoutesWithArgs } from './busterAuthRoutes';
import { BusterSettingsRoutes, BusterSettingsRoutesWithArgs } from './busterSettingsRoutes';

export enum BusterRootRoutes {
  ROOT = '/'
}

export type BusterRootRoutesWithArgs = {
  [BusterRootRoutes.ROOT]: { route: BusterRootRoutes.ROOT };
};

export const BusterRoutes = {
  ...BusterAppRoutes,
  ...BusterAuthRoutes,
  ...BusterSettingsRoutes,
  ...BusterRootRoutes
};

export type BusterRoutes =
  | BusterAppRoutes
  | BusterAuthRoutes
  | BusterSettingsRoutes
  | BusterRootRoutes;

export type BusterRoutesWithArgs = BusterRootRoutesWithArgs &
  BusterAuthRoutesWithArgs &
  BusterAppRoutesWithArgs &
  BusterSettingsRoutesWithArgs;

export type BusterRoutesWithArgsRoute = BusterRoutesWithArgs[BusterRoutes];

export const createBusterRoute = ({ route, ...args }: BusterRoutesWithArgsRoute) => {
  if (!args) return route;
  return Object.entries(args).reduce<string>((acc, [key, value]) => {
    acc.replace(`[${key}]`, value as string);
    return acc.replace(`:${key}`, value as string);
  }, route || '');
};

const routeToRegex = (route: string): RegExp => {
  const dynamicParts = /:[^/]+/g;
  const regexPattern = route.replace(dynamicParts, '[^/]+');
  return new RegExp(`^${regexPattern}$`);
};

const matchDynamicUrlToRoute = (pathname: string): BusterAppRoutes | null => {
  const routes = Object.values(BusterAppRoutes) as string[];
  for (const route of routes) {
    const regex = routeToRegex(route);
    if (regex.test(pathname)) {
      return route as BusterAppRoutes;
    }
  }
  return null;
};

export const createPathnameToBusterRoute = (pathname: string): BusterRoutes => {
  const foundRoute = Object.values(BusterRoutes).find((route) => route === pathname);
  return foundRoute || (matchDynamicUrlToRoute(pathname) as BusterRoutes);
};
