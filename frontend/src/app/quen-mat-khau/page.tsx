'use client';

import { zodResolver } from '@hookform/resolvers/zod';
import { useMutation } from '@tanstack/react-query';
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
import { useI18n } from '@/i18n/provider';
import { authApi, localizedError, type OtpSent } from '@/lib/api';

/** Quên mật khẩu: email → OTP → hệ thống cấp mật khẩu mới qua email. */
export default function ForgotPasswordPage() {
  const { t, locale } = useI18n();
  const router = useRouter();
  const [sent, setSent] = useState<(OtpSent & { email: string }) | null>(null);

  const schema = z.object({ email: z.string().trim().toLowerCase().email(t('auth.v.email')).max(254) });
  const form = useForm<{ email: string }>({ resolver: zodResolver(schema), defaultValues: { email: '' } });

  const requestOtp = useMutation({
    mutationFn: authApi.forgotPassword,
    onSuccess: (result, values) => setSent({ ...result, email: values.email }),
    onError: (error) => toast.error(localizedError(error, t, locale)),
  });
  const reset = useMutation({
    mutationFn: authApi.resetPassword,
    onSuccess: () => {
      toast.success(t('auth.resetDone'), { duration: 8000 });
      router.push('/dang-nhap');
    },
    onError: (error) => toast.error(localizedError(error, t, locale)),
  });

  const error = form.formState.errors.email?.message;

  return (
    <AuthShell title={sent ? t('auth.otpTitle') : t('auth.forgotTitle')} subtitle={t('auth.forgotSubtitle')}>
      {sent ? (
        <OtpStep
          email={sent.email}
          expiresIn={sent.expires_in}
          resendAfter={sent.resend_after}
          submitLabel={t('auth.resetButton')}
          pending={reset.isPending}
          resending={requestOtp.isPending}
          onSubmit={(otp) => reset.mutate({ email: sent.email, otp })}
          onResend={() => requestOtp.mutate({ email: sent.email })}
          onBack={() => setSent(null)}
        />
      ) : (
        <form onSubmit={form.handleSubmit((values) => requestOtp.mutate(values))} className="space-y-5" noValidate>
          <Field id="email" label={t('auth.email')} error={error}>
            <Input id="email" type="email" autoComplete="email" aria-invalid={!!error} {...form.register('email')} />
          </Field>
          <Button type="submit" size="lg" className="w-full" disabled={requestOtp.isPending}>
            {requestOtp.isPending ? <Loader2 className="animate-spin" /> : <Send />} {t('auth.sendOtp')}
          </Button>
          <p className="text-center">
            <Link href="/dang-nhap" className="text-[15px] font-semibold text-primary-700 hover:underline">
              {t('auth.backToLogin')}
            </Link>
          </p>
        </form>
      )}
    </AuthShell>
  );
}
