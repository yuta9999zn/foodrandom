'use client';

import { zodResolver } from '@hookform/resolvers/zod';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { KeyRound, Loader2, Save, UserRound } from 'lucide-react';
import Link from 'next/link';
import { useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { toast } from 'sonner';
import { z } from 'zod';

import { EmptyState } from '@/components/states';
import { Button } from '@/components/ui/button';
import { Field, Input } from '@/components/ui/input';
import { ME_KEY, useMe } from '@/hooks/use-auth';
import { INTL_LOCALE } from '@/i18n/config';
import { useI18n } from '@/i18n/provider';
import { authApi, localizedError } from '@/lib/api';

function ProfileForm({ displayName }: { displayName: string }) {
  const { t, locale } = useI18n();
  const queryClient = useQueryClient();
  const schema = z.object({ display_name: z.string().trim().min(1, t('auth.v.displayName')).max(60, t('auth.v.displayName')) });
  const form = useForm<{ display_name: string }>({ resolver: zodResolver(schema), defaultValues: { display_name: displayName } });
  useEffect(() => form.reset({ display_name: displayName }), [displayName, form]);

  const save = useMutation({
    mutationFn: authApi.updateProfile,
    onSuccess: (user) => {
      queryClient.setQueryData(ME_KEY, user);
      toast.success(t('account.saved'));
    },
    onError: (error) => toast.error(localizedError(error, t, locale)),
  });
  const error = form.formState.errors.display_name?.message;

  return (
    <form onSubmit={form.handleSubmit((v) => save.mutate(v))} className="space-y-4" noValidate>
      <Field id="display_name" label={t('account.displayName')} error={error}>
        <Input id="display_name" aria-invalid={!!error} {...form.register('display_name')} />
      </Field>
      <Button type="submit" disabled={save.isPending}>
        {save.isPending ? <Loader2 className="animate-spin" /> : <Save />} {t('account.save')}
      </Button>
    </form>
  );
}

function PasswordForm() {
  const { t, locale } = useI18n();
  const schema = z
    .object({
      current_password: z.string().min(1, t('auth.v.password')),
      new_password: z.string().min(8, t('auth.v.newPassword')).max(128, t('auth.v.newPassword')),
      confirm: z.string(),
    })
    .refine((v) => v.new_password === v.confirm, { path: ['confirm'], message: t('auth.v.confirm') });
  type Values = z.infer<typeof schema>;
  const form = useForm<Values>({ resolver: zodResolver(schema), defaultValues: { current_password: '', new_password: '', confirm: '' } });

  const change = useMutation({
    mutationFn: (v: Values) => authApi.changePassword({ current_password: v.current_password, new_password: v.new_password }),
    onSuccess: () => {
      toast.success(t('account.passwordChanged'));
      form.reset();
    },
    onError: (error) => toast.error(localizedError(error, t, locale)),
  });
  const { errors } = form.formState;

  return (
    <form onSubmit={form.handleSubmit((v) => change.mutate(v))} className="space-y-4" noValidate>
      <Field id="current_password" label={t('account.currentPassword')} error={errors.current_password?.message}>
        <Input id="current_password" type="password" autoComplete="current-password" {...form.register('current_password')} />
      </Field>
      <Field id="new_password" label={t('account.newPassword')} error={errors.new_password?.message}>
        <Input id="new_password" type="password" autoComplete="new-password" {...form.register('new_password')} />
      </Field>
      <Field id="confirm" label={t('account.confirmPassword')} error={errors.confirm?.message}>
        <Input id="confirm" type="password" autoComplete="new-password" {...form.register('confirm')} />
      </Field>
      <Button type="submit" variant="dark" disabled={change.isPending}>
        {change.isPending ? <Loader2 className="animate-spin" /> : <KeyRound />} {t('account.changePassword')}
      </Button>
    </form>
  );
}

export default function AccountPage() {
  const { t, locale } = useI18n();
  const { data: me, isPending } = useMe();

  if (isPending) return <div className="container py-16"><div className="skeleton mx-auto h-80 max-w-3xl" /></div>;
  if (!me) {
    return (
      <div className="container py-16">
        <EmptyState
          title={t('account.title')}
          description={t('account.needLogin')}
          icon={<UserRound className="size-8" />}
          action={
            <Button asChild>
              <Link href="/dang-nhap?next=/tai-khoan">{t('nav.login')}</Link>
            </Button>
          }
        />
      </div>
    );
  }

  return (
    <div className="container max-w-3xl py-10">
      <div className="flex items-center gap-4">
        <div className="grid size-16 place-items-center rounded-full bg-gradient-to-br from-primary-300 to-primary-600 text-2xl font-bold text-white">
          {me.display_name.charAt(0).toUpperCase()}
        </div>
        <div>
          <h1 className="text-3xl font-bold">{me.display_name}</h1>
          <p className="text-ink-muted">
            {me.email} · {me.phone}
          </p>
          <p className="text-sm text-ink-soft">
            {t('account.memberSince', { date: new Date(me.created_at).toLocaleDateString(INTL_LOCALE[locale]) })}
          </p>
        </div>
      </div>
      <div className="mt-8 grid gap-6 md:grid-cols-2">
        <section className="rounded-3xl bg-surface p-6 shadow-card ring-1 ring-border">
          <h2 className="mb-4 text-xl font-bold">{t('account.profile')}</h2>
          <ProfileForm displayName={me.display_name} />
        </section>
        <section className="rounded-3xl bg-surface p-6 shadow-card ring-1 ring-border">
          <h2 className="mb-4 text-xl font-bold">{t('account.changePassword')}</h2>
          <PasswordForm />
        </section>
      </div>
    </div>
  );
}
