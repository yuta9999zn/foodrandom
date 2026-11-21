'use client';

import { useMutation } from '@tanstack/react-query';
import { Loader2, LocateFixed, MapPin, Search } from 'lucide-react';
import { useState, type FormEvent } from 'react';
import { toast } from 'sonner';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { usePrefs } from '@/hooks/use-favorites';
import { useGeolocation } from '@/hooks/use-geolocation';
import { useI18n } from '@/i18n/provider';
import { localizedError, locationApi } from '@/lib/api';
import type { GeoPoint } from '@/lib/types';
import { useLocationStore } from '@/stores/location-store';

/**
 * Chọn vị trí: dùng định vị của trình duyệt hoặc nhập địa chỉ thủ công (tìm trong quốc gia đang chọn).
 * Người dùng từ chối quyền vị trí thì vẫn tiếp tục được bằng ô nhập địa chỉ.
 */
export function LocationPicker({ onPicked }: { onPicked?: () => void }) {
  const { t, locale, market } = useI18n();
  const prefs = usePrefs();
  const setLocation = useLocationStore((s) => s.setLocation);
  const { locate, locating, geoErrorMessage } = useGeolocation();
  const [address, setAddress] = useState('');
  const [geoMessage, setGeoMessage] = useState<string | null>(null);

  const geocode = useMutation({ mutationFn: (q: string) => locationApi.geocode(q, prefs) });

  const handleLocate = async () => {
    setGeoMessage(null);
    try {
      const location = await locate();
      setLocation({ ...location, market });
      toast.success(t('filter.gotLocation', { label: location.label }));
      onPicked?.();
    } catch (error) {
      setGeoMessage(geoErrorMessage(error));
    }
  };

  const handleSubmit = (event: FormEvent) => {
    event.preventDefault();
    const q = address.trim();
    if (q.length < 2) {
      toast.error(t('filter.minChars'));
      return;
    }
    geocode.mutate(q);
  };

  const pick = (point: GeoPoint) => {
    setLocation({ lat: point.lat, lng: point.lng, label: point.label, source: 'manual', market });
    geocode.reset();
    onPicked?.();
  };

  return (
    <div className="space-y-4">
      <Button className="w-full" size="lg" onClick={handleLocate} disabled={locating}>
        {locating ? <Loader2 className="animate-spin" /> : <LocateFixed />}
        {locating ? t('nearby.locating') : t('nearby.allow')}
      </Button>

      {geoMessage && (
        <p role="alert" className="rounded-2xl bg-secondary/15 px-4 py-3 text-sm font-medium text-ink">
          {geoMessage}
        </p>
      )}

      <div className="flex items-center gap-3 text-sm text-ink-soft">
        <span className="h-px flex-1 bg-border" /> {t('nearby.orAddress')} <span className="h-px flex-1 bg-border" />
      </div>

      <form onSubmit={handleSubmit} className="flex gap-2">
        <Input
          value={address}
          onChange={(e) => setAddress(e.target.value)}
          placeholder={t('filter.locationPlaceholder', { country: t(`market.names.${market}`) })}
          aria-label={t('nearby.addressAria')}
          maxLength={200}
        />
        <Button type="submit" variant="dark" className="h-12 shrink-0" disabled={geocode.isPending}>
          {geocode.isPending ? <Loader2 className="animate-spin" /> : <Search />} {t('nearby.find')}
        </Button>
      </form>

      {geocode.isError && (
        <p role="alert" className="text-sm font-medium text-primary-700">
          {localizedError(geocode.error, t, locale)}
        </p>
      )}

      {geocode.data && (
        <div>
          {geocode.data.is_mock && <p className="mb-2 rounded-xl bg-secondary/20 px-3 py-2 text-sm font-semibold">{t('filter.mockAddress')}</p>}
          {geocode.data.results.length === 0 ? (
            <p className="text-sm text-ink-muted">{t('filter.addressNotFound')}</p>
          ) : (
            <ul className="divide-y divide-border overflow-hidden rounded-2xl border border-border">
              {geocode.data.results.map((point) => (
                <li key={`${point.lat},${point.lng}`}>
                  <button
                    type="button"
                    onClick={() => pick(point)}
                    className="flex w-full items-start gap-2.5 px-3.5 py-3 text-left text-[15px] transition hover:bg-primary-50"
                  >
                    <MapPin className="mt-0.5 size-[18px] shrink-0 text-primary" />
                    <span>{point.label}</span>
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
    </div>
  );
}
