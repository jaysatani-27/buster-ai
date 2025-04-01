import { AppContentHeader } from '../_components/AppContentHeader';
import { DashboardHeader } from './_DashboardHeader';
import { DashboardListContent } from './_DashboardListContent';

export default function DashboardPage(props: any) {
  return (
    <>
      <DashboardHeader />
      <DashboardListContent />
    </>
  );
}
