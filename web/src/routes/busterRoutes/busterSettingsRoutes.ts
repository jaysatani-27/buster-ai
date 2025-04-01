export enum BusterSettingsRoutes {
  SETTINGS = '/app/settings'
}

export type BusterSettingsRoutesWithArgs = {
  [BusterSettingsRoutes.SETTINGS]: { route: BusterSettingsRoutes.SETTINGS };
};
