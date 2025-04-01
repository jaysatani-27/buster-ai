import { Trendline } from '@/components/charts/interfaces';
import { LoopTrendline } from './EditTrendline';

export const trendlineOptions: {
  label: string;
  value: LoopTrendline['type'];
  disabled?: boolean;
}[] = [
  { label: 'Average', value: 'average' },
  { label: 'Linear', value: 'linear_regression' },
  { label: 'Logarithmic', value: 'logarithmic_regression' },
  { label: 'Exponential', value: 'exponential_regression' },
  { label: 'Polynomial', value: 'polynomial_regression' },
  { label: 'Min', value: 'min' },
  { label: 'Max', value: 'max' },
  { label: 'Median', value: 'median' }
];

export const TypeToLabel: Record<Trendline['type'], string> = {
  linear_regression: 'Linear',
  logarithmic_regression: 'Logarithmic',
  exponential_regression: 'Exponential',
  polynomial_regression: 'Polynomial',
  min: 'Min',
  max: 'Max',
  median: 'Median',
  average: 'Average'
};
