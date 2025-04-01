import { useCreateReactQuery } from '@/api/createReactQuery';
import { listAllGoogleFontsFromNext } from './requests';

export enum NEXT_GOOGLE_QUERY_KEYS {
  getGoogleFontsFromNext = 'getGoogleFontsFromNext'
}

export const useGetGoogleFontsFromNext = ({ enabled = true }: { enabled?: boolean }) => {
  return useCreateReactQuery({
    queryKey: [NEXT_GOOGLE_QUERY_KEYS.getGoogleFontsFromNext],
    queryFn: listAllGoogleFontsFromNext,
    refetchOnWindowFocus: false,
    refetchOnMount: false,
    enabled
  });
};
