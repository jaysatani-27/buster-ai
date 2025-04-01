import { AnimationSpec } from 'chart.js';

export interface CustomAnimationSpec extends AnimationSpec<'doughnut' | 'pie'> {
  onProgress?: (animation: { currentStep: number; numSteps: number }) => void;
}
