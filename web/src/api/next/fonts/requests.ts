import { GoogleFont } from '../../other/google/interfaces';
import { nextApi } from '../instances';

export const listAllGoogleFontsFromNext = async () => {
  return nextApi.get('/api/fonts').then(async ({ data }) => {
    return data.items as GoogleFont[];
  });
};
