'use client';

import Link from 'next/link';

import { EmptyState } from '@/components/states';
import { Button } from '@/components/ui/button';
import { useI18n } from '@/i18n/provider';

export default function NotFound() {
  const { t } = useI18n();
  return (
    <div className="container py-20">
      <EmptyState
        title={t('states.notFoundTitle')}
        description={t('states.notFoundDesc')}
        action={
          <Button asChild>
            <Link href="/">{t('states.backHome')}</Link>
          </Button>
        }
      />
    </div>
  );
}
