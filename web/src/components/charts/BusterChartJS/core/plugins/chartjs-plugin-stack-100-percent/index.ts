import { beforeInit, beforeUpdate, afterUpdate } from './plugin';
export * from './types';

export const ChartPluginStacked100 = {
  id: 'stacked100',
  beforeInit,
  beforeUpdate,
  afterUpdate
};

export default ChartPluginStacked100;
