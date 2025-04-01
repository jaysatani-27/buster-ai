import { useCreateReactQuery } from '@/api/createReactQuery';
import { listAllGoogleFontsFromGoogle } from './requests';

export enum GOOGLE_QUERY_KEYS {
  getGoogleFonts = 'getGoogleFonts'
}

export const useGetGoogleFonts = () => {
  const d = useCreateReactQuery({
    queryKey: [GOOGLE_QUERY_KEYS.getGoogleFonts],
    queryFn: listAllGoogleFontsFromGoogle,
    refetchOnWindowFocus: false,
    refetchOnMount: false
  });

  return d;
};
