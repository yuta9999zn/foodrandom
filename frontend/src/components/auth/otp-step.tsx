'use client';

import { zodResolver } from '@hookform/resolvers/zod';
import { Loader2, MailCheck } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useForm } from 'react-hook-form';
import { z } from 'zod';

import { Button } from '@/components/ui/button';
import { Field, Input } from '@/components/ui/input';
import { useI18n } from '@/i18n/provider';

/** Bước nhập mã OTP 6 chữ số, có đếm ngược gửi lại mã. */
export function OtpStep({
  email,
  expiresIn,
  resendAfter,
  submitLabel,
  pending,
  resending,
  onSubmit,
  onResend,
  onBack,
}: {
  email: string;
  expiresIn: number;
  resendAfter: number;
  submitLabel: string;
  pending: boolean;
  resending: boolean;
  onSubmit: (otp: string) => void;
  onResend: () => void;
  onBack: () => void;
}) {
  const { t } = useI18n();
  const [countdown, setCountdown] = useState(resendAfter);

  useEffect(() => setCountdown(resendAfter), [resendAfter, email]);
  useEffect(() => {
    if (countdown <= 0) return;
    const timer = setTimeout(() => setCountdown((c) => c - 1), 1000);
    return () => clearTimeout(timer);
  }, [countdown]);

  const schema = z.object({ otp: z.string().regex(/^\d{6}$/, t('auth.v.otp')) });
  const form = useForm<{ otp: string }>({ resolver: zodResolver(schema), defaultValues: { otp: '' } });
  const error = form.formState.errors.otp?.message;

  return (
    <form onSubmit={form.handleSubmit(({ otp }) => onSubmit(otp))} className="space-y-5" noValidate>
      <div className="flex gap-3 rounded-2xl bg-leaf-soft px-4 py-3 text-[15px] text-ink">
        <MailCheck className="mt-0.5 size-5 shrink-0 text-leaf" />
        <p>{t('auth.otpSent', { email, minutes: Math.round(expiresIn / 60) })}</p>
      </div>
      <Field id="otp" label={t('auth.otp')} error={error}>
        <Input
          id="otp"
          inputMode="numeric"
          autoComplete="one-time-code"
          maxLength={6}
          autoFocus
          aria-invalid={!!error}
          className="text-center font-mono text-2xl tracking-[0.5em]"
          {...form.register('otp')}
        />
      </Field>
      <Button type="submit" size="lg" className="w-full" disabled={pending}>
        {pending && <Loader2 className="animate-spin" />} {submitLabel}
      </Button>
      <div className="flex items-center justify-between text-sm">
        <button type="button" onClick={onBack} className="font-semibold text-ink-muted hover:text-ink">
          {t('auth.changeEmail')}
        </button>
        <button
          type="button"
          onClick={onResend}
          disabled={countdown > 0 || resending}
          className="font-semibold text-primary-700 hover:underline disabled:cursor-not-allowed disabled:text-ink-soft disabled:no-underline"
        >
          {countdown > 0 ? t('auth.resendIn', { s: countdown }) : t('auth.resend')}
        </button>
      </div>
      {process.env.NODE_ENV !== 'production' && <p className="text-center text-xs text-ink-soft">{t('auth.devMailHint')}</p>}
    </form>
  );
}
