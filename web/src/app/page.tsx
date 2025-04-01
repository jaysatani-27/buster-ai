import { useSupabaseServerContext } from '@/context/Supabase/useSupabaseContext';
import { BusterRoutes, createBusterRoute } from '@/routes';
import { redirect } from 'next/navigation';

export default async function Index() {
  const { user } = await useSupabaseServerContext();

  if (!user) {
    return redirect(
      createBusterRoute({
        route: BusterRoutes.AUTH_LOGIN
      })
    );
  }

  if (user?.id) {
    return redirect(
      createBusterRoute({
        route: BusterRoutes.APP_THREAD
      })
    );
  }

  return <></>;
}
