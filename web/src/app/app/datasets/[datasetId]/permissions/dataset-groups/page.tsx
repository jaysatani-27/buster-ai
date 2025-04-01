import { PermissionDatasetGroups } from './_PermissionDatasetGroups';

export default async function Page({ params }: { params: { datasetId: string } }) {
  return <PermissionDatasetGroups datasetId={params.datasetId} />;
}
