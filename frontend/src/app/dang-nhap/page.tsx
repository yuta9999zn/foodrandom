'use client';

import { zodResolver } from '@hookform/resolvers/zod';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { Loader2, LogIn } from 'lucide-react';
import Link from 'next/link';
import { useRouter, useSearchParams } from 'next/navigation';
import { Suspense } from 'react';
import { useForm } from 'react-hook-form';
import { toast } from 'sonner';
import { z } from 'zod';

import { AuthShell } from '@/components/auth/auth-shell';
import { Button } from '@/components/ui/button';
import { Field, Input } from '@/components/ui/input';
import { ME_KEY } from '@/hooks/use-auth';
import { useI18n } from '@/i18n/provider';
import { authApi, localizedError } from '@/lib/api';

/** Chỉ cho quay lại đường dẫn nội bộ, tránh open redirect. */
function safeNext(value: string | null) {
  return value && value.startsWith('/') && !value.startsWith('//') ? value : '/';
}

function LoginForm() {
  const { t, locale } = useI18n();
  const router = useRouter();
  const params = useSearchParams();
  const queryClient = useQueryClient();

  const schema = z.object({
    identifier: z.string().trim().min(3, t('auth.v.identifier')).max(254),
    password: z.string().min(1, t('auth.v.password')).max(128),
  });
  type Values = z.infer<typeof schema>;
  const form = useForm<Values>({ resolver: zodResolver(schema), defaultValues: { identifier: '', password: '' } });

  const login = useMutation({
    mutationFn: authApi.login,
    onSuccess: (result) => {
      queryClient.setQueryData(ME_KEY, result.user);
      queryClient.invalidateQueries({ queryKey: ['favorites'] });
      toast.success(t('auth.loginSuccess'));
      router.push(safeNext(params.get('next')));
      router.refresh();
    },
    onError: (error) => toast.error(localizedError(error, t, locale)),
  });

  const { errors } = form.formState;
  return (
    <form onSubmit={form.handleSubmit((values) => login.mutate(values))} className="space-y-5" noValidate>
      <Field id="identifier" label={t('auth.identifier')} error={errors.identifier?.message}>
        <Input id="identifier" autoComplete="username" aria-invalid={!!errors.identifier} {...form.register('identifier')} />
      </Field>
      <Field id="password" label={t('auth.password')} error={errors.password?.message}>
        <Input id="password" type="password" autoComplete="current-password" aria-invalid={!!errors.password} {...form.register('password')} />
      </Field>
      <div className="flex justify-end">
        <Link href="/quen-mat-khau" className="text-sm font-semibold text-primary-700 hover:underline">
          {t('auth.forgot')}
        </Link>
      </div>
      <Button type="submit" size="lg" className="w-full" disabled={login.isPending}>
        {login.isPending ? <Loader2 className="animate-spin" /> : <LogIn />} {t('auth.loginButton')}
      </Button>
      <p className="text-center text-[15px] text-ink-muted">
        {t('auth.noAccount')}{' '}
        <Link href="/dang-ky" className="font-semibold text-primary-700 hover:underline">
          {t('auth.registerNow')}
        </Link>
      </p>
    </form>
  );
}

export default function LoginPage() {
  const { t } = useI18n();
  return (
    <AuthShell title={t('auth.loginTitle')} subtitle={t('auth.loginSubtitle')}>
      <Suspense>
        <LoginForm />
      </Suspense>
    </AuthShell>
  );
}
