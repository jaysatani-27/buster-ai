import { PermissionOverview } from './_PermissionOverview';

export default async function Page({ params }: { params: { datasetId: string } }) {
  return <PermissionOverview datasetId={params.datasetId} />;
}
