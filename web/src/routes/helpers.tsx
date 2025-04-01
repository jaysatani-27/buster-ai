import { BusterRoutes, createBusterRoute } from './busterRoutes';

export const pathNameToRoute = (pathName: string, params: any): BusterRoutes => {
  const route = Object.values(BusterRoutes).find((r) => {
    return r === pathName || createBusterRoute({ route: r, ...params }) === pathName;
  });

  const paramRoutesToParent: Record<string, BusterRoutes> = {
    [BusterRoutes.APP_THREAD_ID]: BusterRoutes.APP_THREAD,
    [BusterRoutes.APP_DASHBOARD_THREADS_ID]: BusterRoutes.APP_DASHBOARDS,
    [BusterRoutes.APP_DASHBOARD_ID]: BusterRoutes.APP_DASHBOARDS,
    [BusterRoutes.APP_COLLECTIONS_ID]: BusterRoutes.APP_COLLECTIONS,
    [BusterRoutes.APP_DATASETS_ID]: BusterRoutes.APP_DATASETS,
    [BusterRoutes.APP_DATASETS_ID_PERMISSIONS_OVERVIEW]: BusterRoutes.APP_DATASETS,
    [BusterRoutes.APP_DATASETS_ID_OVERVIEW]: BusterRoutes.APP_DATASETS,
    [BusterRoutes.APP_DATASETS_ID_EDITOR]: BusterRoutes.APP_DATASETS,
    [BusterRoutes.APP_TERMS_ID]: BusterRoutes.APP_TERMS,
    [BusterRoutes.APP_SETTINGS_USERS_ID]: BusterRoutes.APP_SETTINGS_USERS
  };
  if (route && paramRoutesToParent[route as string]) {
    return paramRoutesToParent[route as string];
  }

  return route || BusterRoutes.ROOT;
};
