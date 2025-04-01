export enum Priority {
  HIGH = 'HIGH',
  NORMAL = 'NORMAL'
}

// Add routes that need immediate processing
const HIGH_PRIORITY_ROUTES = ['/threads/list', '/threads/update', '/metrics/list'];

export const getPriorityFromRoute = (route: string): Priority => {
  return HIGH_PRIORITY_ROUTES.some((highPriorityRoute) => route?.startsWith?.(highPriorityRoute))
    ? Priority.HIGH
    : Priority.NORMAL;
};
