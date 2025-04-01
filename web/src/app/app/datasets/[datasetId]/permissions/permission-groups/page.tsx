import { PermissionPermissionGroup } from './_PermissionPermissionGroup';

export default async function Page({ params }: { params: { datasetId: string } }) {
  return <PermissionPermissionGroup datasetId={params.datasetId} />;
}
