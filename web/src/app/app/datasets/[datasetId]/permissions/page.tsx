import { permanentRedirect } from 'next/navigation';
import { BusterRoutes, createBusterRoute } from '@/routes';

export default async function Page({ params: { datasetId } }: { params: { datasetId: string } }) {
  return permanentRedirect(
    createBusterRoute({
      route: BusterRoutes.APP_DATASETS_ID_PERMISSIONS_OVERVIEW,
      datasetId
    })
  );
}
