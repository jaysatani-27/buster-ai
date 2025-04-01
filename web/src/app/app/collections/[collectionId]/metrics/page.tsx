import { createBusterRoute, BusterRoutes } from '@/routes';
import { permanentRedirect } from 'next/navigation';

export default function Metrics() {
  return permanentRedirect(createBusterRoute({ route: BusterRoutes.APP_COLLECTIONS }));
}
