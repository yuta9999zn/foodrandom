'use client';

import { zodResolver } from '@hookform/resolvers/zod';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { Loader2, Send } from 'lucide-react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { toast } from 'sonner';
import { z } from 'zod';

import { AuthShell } from '@/components/auth/auth-shell';
import { OtpStep } from '@/components/auth/otp-step';
import { Button } from '@/components/ui/button';
import { Field, Input } from '@/components/ui/input';
import { ME_KEY } from '@/hooks/use-auth';
import { useI18n } from '@/i18n/provider';
import { ApiError, authApi, localizedError, type OtpSent } from '@/lib/api';

/** Đăng ký: email + số điện thoại → OTP về email → hệ thống cấp mật khẩu và gửi về email. */
export default function RegisterPage() {
  const { t, locale } = useI18n();
  const router = useRouter();
  const queryClient = useQueryClient();
  const [sent, setSent] = useState<(OtpSent & { email: string; phone: string }) | null>(null);

  const schema = z.object({
    email: z.string().trim().toLowerCase().email(t('auth.v.email')).max(254),
    phone: z
      .string()
      .trim()
      .regex(/^(\+?84|0)(3|5|7|8|9)\d{8}$/, t('auth.v.phone')),
  });
  type Values = z.infer<typeof schema>;
  const form = useForm<Values>({ resolver: zodResolver(schema), defaultValues: { email: '', phone: '' } });

  const requestOtp = useMutation({
    mutationFn: authApi.requestRegisterOtp,
    onSuccess: (result, values) => setSent({ ...result, ...values }),
    onError: (error) => {
      if (error instanceof ApiError && error.code === 'VALIDATION_ERROR') {
        for (const field of ['email', 'phone'] as const) {
          const message = error.field(field);
          if (message) form.setError(field, { message: locale === 'vi' ? message : t(`auth.v.${field}`) });
        }
      }
      toast.error(localizedError(error, t, locale));
    },
  });

  const verify = useMutation({
    mutationFn: authApi.verifyRegister,
    onSuccess: (result) => {
      queryClient.setQueryData(ME_KEY, result.user);
      toast.success(locale === 'vi' ? result.message : t('auth.registered'), { duration: 8000 });
      router.push('/');
      router.refresh();
    },
    onError: (error) => toast.error(localizedError(error, t, locale)),
  });

  const { errors } = form.formState;

  return (
    <AuthShell title={sent ? t('auth.otpTitle') : t('auth.registerTitle')} subtitle={t('auth.registerSubtitle')}>
      {sent ? (
        <OtpStep
          email={sent.email}
          expiresIn={sent.expires_in}
          resendAfter={sent.resend_after}
          submitLabel={t('auth.verify')}
          pending={verify.isPending}
          resending={requestOtp.isPending}
          onSubmit={(otp) => verify.mutate({ email: sent.email, otp })}
          onResend={() => requestOtp.mutate({ email: sent.email, phone: sent.phone })}
          onBack={() => setSent(null)}
        />
      ) : (
        <form onSubmit={form.handleSubmit((values) => requestOtp.mutate(values))} className="space-y-5" noValidate>
          <Field id="email" label={t('auth.email')} error={errors.email?.message}>
            <Input id="email" type="email" autoComplete="email" aria-invalid={!!errors.email} {...form.register('email')} />
          </Field>
          <Field id="phone" label={t('auth.phone')} error={errors.phone?.message} hint={t('auth.phoneHint')}>
            <Input id="phone" type="tel" autoComplete="tel" aria-invalid={!!errors.phone} {...form.register('phone')} />
          </Field>
          <Button type="submit" size="lg" className="w-full" disabled={requestOtp.isPending}>
            {requestOtp.isPending ? <Loader2 className="animate-spin" /> : <Send />} {t('auth.sendOtp')}
          </Button>
          <p className="text-center text-[15px] text-ink-muted">
            {t('auth.haveAccount')}{' '}
            <Link href="/dang-nhap" className="font-semibold text-primary-700 hover:underline">
              {t('nav.login')}
            </Link>
          </p>
        </form>
      )}
    </AuthShell>
  );
}
