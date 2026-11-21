'use client';

import { ErrorState } from '@/components/states';
import { useI18n } from '@/i18n/provider';

export default function Error({ reset }: { error: Error & { digest?: string }; reset: () => void }) {
  const { t } = useI18n();
  return (
    <div className="container py-20">
      <ErrorState title={t('states.errorTitle')} message={t('states.errorDesc')} retryLabel={t('states.retry')} onRetry={reset} />
    </div>
  );
}
