import { PermissionUsers } from './_PermissionUsers';

export default async function Page({ params }: { params: { datasetId: string } }) {
  return <PermissionUsers datasetId={params.datasetId} />;
}
