'use client';

import { Radar } from 'lucide-react';
import { useEffect, useState } from 'react';

import { useI18n } from '@/i18n/provider';
import { cn } from '@/lib/utils';
import { MAX_RADIUS_KM, MIN_RADIUS_KM, RADIUS_PRESETS, useLocationStore } from '@/stores/location-store';

/**
 * Chọn bán kính tìm quán: các mức nhanh, thanh kéo 1–50 km để chọn số km bất kỳ, hoặc "Không giới hạn".
 * Thanh kéo chỉ áp dụng khi thả tay (hoặc sau khi ngừng bấm phím) để không gọi API liên tục.
 */
export function RadiusPicker({ className }: { className?: string }) {
  const { t } = useI18n();
  const radius = useLocationStore((s) => s.radius);
  const setRadius = useLocationStore((s) => s.setRadius);
  const [draft, setDraft] = useState<number>(radius ?? MAX_RADIUS_KM);

  useEffect(() => setDraft(radius ?? MAX_RADIUS_KM), [radius]);

  // Phím mũi tên: áp dụng sau khi ngừng bấm 400 ms.
  useEffect(() => {
    if (radius != null && draft === radius) return;
    if (radius == null && draft === MAX_RADIUS_KM) return;
    const timer = setTimeout(() => setRadius(draft), 400);
    return () => clearTimeout(timer);
  }, [draft, radius, setRadius]);

  const chip = (active: boolean) =>
    cn(
      'shrink-0 rounded-full px-3 py-1.5 text-sm font-semibold transition',
      active ? 'chip-active' : 'bg-surface text-ink ring-1 ring-border hover:bg-primary-50',
    );
  const percent = ((draft - MIN_RADIUS_KM) / (MAX_RADIUS_KM - MIN_RADIUS_KM)) * 100;

  return (
    <div className={cn('rounded-2xl bg-background p-3.5 ring-1 ring-border', className)} role="group" aria-label={t('nearby.radiusAria')}>
      <div className="flex items-center justify-between gap-3">
        <span className="flex items-center gap-1.5 text-sm font-semibold">
          <Radar className="size-4 text-primary" /> {t('nearby.radiusLabel')}
        </span>
        <span className="rounded-full bg-primary-50 px-2.5 py-0.5 text-sm font-bold text-primary-700" aria-live="polite">
          {radius == null ? t('options.radius.none') : t('options.radius.within', { km: draft })}
        </span>
      </div>

      <div className="mt-3 flex gap-1.5 overflow-x-auto pb-0.5 scrollbar-none">
        {RADIUS_PRESETS.map((km) => (
          <button key={km} type="button" aria-pressed={radius === km} onClick={() => setRadius(km)} className={chip(radius === km)}>
            {km} km
          </button>
        ))}
        <button type="button" aria-pressed={radius == null} onClick={() => setRadius(null)} className={chip(radius == null)}>
          {t('options.radius.none')}
        </button>
      </div>

      <label className="mt-3 block">
        <span className="sr-only">{t('nearby.radiusSlider')}</span>
        <input
          type="range"
          min={MIN_RADIUS_KM}
          max={MAX_RADIUS_KM}
          step={1}
          value={draft}
          onChange={(e) => setDraft(Number(e.target.value))}
          onPointerUp={() => setRadius(draft)}
          aria-valuetext={t('options.radius.within', { km: draft })}
          className="h-2 w-full cursor-pointer appearance-none rounded-full accent-primary"
          style={{ background: `linear-gradient(to right, #E85D78 ${percent}%, #EAD5D1 ${percent}%)` }}
        />
        <span className="mt-1 flex justify-between text-xs text-ink-soft" aria-hidden="true">
          <span>{MIN_RADIUS_KM} km</span>
          <span>{t('nearby.radiusSlider')}</span>
          <span>{MAX_RADIUS_KM} km</span>
        </span>
      </label>
    </div>
  );
}
