import { LoginForm } from './_components/LoginForm';
import { useBusterSupabaseAuthMethods } from '@/hooks/useBusterSupabaseAuthMethods';
import { redirect } from 'next/navigation';
import { BusterRoutes, createBusterRoute } from '@/routes';
import { useSupabaseServerContext } from '@/context/Supabase/useSupabaseContext';

export default async function Login({}: {}) {
  const supabase = await useSupabaseServerContext();
  const { user } = supabase;
  const {
    signInWithEmailAndPassword,
    signInWithGoogle,
    signInWithGithub,
    signInWithAzure,
    signUp
  } = useBusterSupabaseAuthMethods();
  if (user?.id) {
    return redirect(
      createBusterRoute({
        route: BusterRoutes.APP_ROOT
      })
    );
  }

  return (
    <LoginForm
      user={user}
      signInWithEmailAndPassword={signInWithEmailAndPassword}
      signInWithGoogle={signInWithGoogle}
      signInWithGithub={signInWithGithub}
      signInWithAzure={signInWithAzure}
      signUp={signUp}
    />
  );
}
