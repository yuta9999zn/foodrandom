'use client';

import { MapPin } from 'lucide-react';
import type { ComponentProps } from 'react';

import { Button } from '@/components/ui/button';
import { useI18n } from '@/i18n/provider';
import { mapsSearchUrl } from '@/lib/maps';
import { cn } from '@/lib/utils';

type ButtonProps = Pick<ComponentProps<typeof Button>, 'variant' | 'size' | 'className'>;

/**
 * Nút mở Google Maps (tab mới / ứng dụng Google Maps trên điện thoại) với tên món trong ô tìm kiếm.
 * Người dùng tự tìm và chọn quán gần mình nhất ngay trong Google Maps.
 */
export function MapsSearchButton({ name, label, ...props }: ButtonProps & { name: string; label?: string }) {
  const { t } = useI18n();
  return (
    <Button asChild {...props}>
      <a href={mapsSearchUrl(name)} target="_blank" rel="noopener noreferrer" aria-label={t('maps.openFor', { name })}>
        <MapPin /> {label ?? t('maps.open')}
      </a>
    </Button>
  );
}

/** Nút tròn nhỏ trên thẻ món: mở Google Maps tìm món này. */
export function MapsSearchIcon({ name, className }: { name: string; className?: string }) {
  const { t } = useI18n();
  return (
    <a
      href={mapsSearchUrl(name)}
      target="_blank"
      rel="noopener noreferrer"
      aria-label={t('maps.openFor', { name })}
      title={t('maps.open')}
      className={cn(
        'relative z-10 grid size-11 shrink-0 place-items-center rounded-full bg-primary-50 text-primary transition hover:bg-primary hover:text-white',
        className,
      )}
    >
      <MapPin className="size-[22px] fill-current [&>circle]:fill-white" />
    </a>
  );
}
