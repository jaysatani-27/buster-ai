import { getMyUserInfo } from '@/api/buster_rest/users/requests';
import { useSupabaseServerContext } from '@/context/Supabase/useSupabaseContext';
import { checkIfUserIsAdmin } from '@/context/Users/helpers';

export const useCheckIfUserIsAdmin_server = async (): Promise<boolean> => {
  const supabaseContext = await useSupabaseServerContext();
  const userInfo = await getMyUserInfo({ jwtToken: supabaseContext.accessToken });
  const isAdmin = checkIfUserIsAdmin(userInfo);

  return isAdmin;
};
