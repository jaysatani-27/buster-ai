import { BarAndLineAxis } from '../../../interfaces';
import { appendToKeyValueChain, extractFieldsFromChain } from '../../../chartHooks';
import first from 'lodash/first';

export const getYAxisColumnNames = (y: string, selectedAxisY: string[]): string[] => {
  return extractFieldsFromChain(y) //bar roundess with category is why I need to pass true
    .filter(({ key }) => selectedAxisY.includes(key))
    .map(({ key }) => key);
};

export const seriesNameGenerator = (
  y: string,
  hasMultipleMeasures: boolean,
  hasMultipleCategories: boolean
) => {
  if (!hasMultipleMeasures && !hasMultipleCategories) {
    const firstItem = first(extractFieldsFromChain(y))!;
    return appendToKeyValueChain([firstItem]);
  }
  return y;
};
