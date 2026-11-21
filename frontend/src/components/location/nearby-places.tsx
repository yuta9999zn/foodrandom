'use client';

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Crown, ExternalLink, Heart, Info, MapPin, Navigation, Star, TriangleAlert } from 'lucide-react';
import { useState } from 'react';
import { toast } from 'sonner';

import { Button } from '@/components/ui/button';
import { useMe } from '@/hooks/use-auth';
import { usePrefs, useRequireLogin } from '@/hooks/use-favorites';
import { useI18n } from '@/i18n/provider';
import { favoritesApi, localizedError, locationApi } from '@/lib/api';
import { directionsUrl, searchUrl } from '@/lib/maps';
import type { Place } from '@/lib/types';
import { cn, formatCount, formatDistance } from '@/lib/utils';
import { MAX_RADIUS_KM, RADIUS_PRESETS, radiusKm, useLocationStore } from '@/stores/location-store';
import type { NearbyTarget } from '@/stores/nearby-dialog-store';

import { RadiusPicker } from './radius-picker';

const PLACES_KEY = ['favorites', 'places'] as const;

/** Lưu / bỏ lưu quán yêu thích. Khách được mời đăng nhập. */
function useSavedPlaces() {
  const { data: me } = useMe();
  const queryClient = useQueryClient();
  const requireLogin = useRequireLogin();
  const { t, locale } = useI18n();

  const saved = useQuery({
    queryKey: PLACES_KEY,
    queryFn: async () => (await favoritesApi.places()).items,
    enabled: !!me,
    staleTime: 60_000,
  });

  const toggle = useMutation({
    mutationFn: async ({ place, target, savedId }: { place: Place; target: NearbyTarget; savedId?: string }) => {
      if (savedId) {
        await favoritesApi.removePlace(savedId);
        return false;
      }
      await favoritesApi.savePlace(place, { cuisine: target.cuisine, food_slug: target.slug });
      return true;
    },
    onSuccess: (nowSaved) => {
      toast.success(nowSaved ? t('nearby.saved') : t('nearby.unsaved'));
      queryClient.invalidateQueries({ queryKey: PLACES_KEY });
    },
    onError: (error) => toast.error(localizedError(error, t, locale)),
  });

  const savedId = (place: Place) =>
    saved.data?.find((p) => p.provider === place.provider && p.place_ref === place.place_ref)?.id;

  return {
    savedId,
    toggle: (place: Place, target: NearbyTarget) => {
      if (!requireLogin('actionSavePlace')) return;
      toggle.mutate({ place, target, savedId: savedId(place) });
    },
  };
}

/** Danh sách quán gần vị trí người dùng phục vụ một món, kèm nút chỉ đường mở Google Maps. */
export function NearbyPlaces({ target, limit = 5, compact }: { target: NearbyTarget; limit?: number; compact?: boolean }) {
  const { t, locale, formatNumber } = useI18n();
  const prefs = usePrefs();
  const location = useLocationStore((s) => s.location);
  const radius = useLocationStore((s) => s.radius);
  const setRadius = useLocationStore((s) => s.setRadius);
  const { savedId, toggle } = useSavedPlaces();
  // Mức kế tiếp để gợi ý "Mở rộng lên … km" khi không có quán trong bán kính hiện tại.
  const widerRadius = radius == null ? null : ([...RADIUS_PRESETS, MAX_RADIUS_KM] as number[]).find((km) => km > radius) ?? null;
  // Mặc định: vài quán gần nhất, xếp theo số sao Google cao nhất.
  const [sort, setSort] = useState<'rating' | 'distance'>('rating');

  const query = useQuery({
    queryKey: ['nearby', target.slug, location?.lat.toFixed(4), location?.lng.toFixed(4), radius, limit, sort, prefs],
    queryFn: () =>
      locationApi.nearby({ lat: location!.lat, lng: location!.lng, radius_km: radiusKm(radius), food: target.slug, limit, sort }, prefs),
    enabled: !!location,
    staleTime: 5 * 60_000,
    retry: false,
  });

  if (!location) return null;

  const mapsSearch = (
    <Button asChild variant="outline" className="w-full">
      <a href={searchUrl(target.name, location, radius)} target="_blank" rel="noopener noreferrer">
        <ExternalLink /> {t('nearby.searchOnMaps', { name: target.name })}
      </a>
    </Button>
  );

  if (query.isPending) {
    return (
      <div className="space-y-3" aria-busy="true" aria-label={t('nearby.loading')}>
        <RadiusPicker />
        {Array.from({ length: compact ? 2 : 3 }, (_, i) => (
          <div key={i} className="skeleton h-[84px]" />
        ))}
      </div>
    );
  }

  if (query.isError) {
    // Nhà cung cấp địa điểm lỗi: vẫn cho người dùng đi tiếp bằng Google Maps (tìm món quanh vị trí của họ, ở bất kỳ quốc gia nào).
    return (
      <div className="space-y-3">
        <RadiusPicker />
        <Button asChild size="lg" className="w-full">
          <a href={searchUrl(target.name, location, radius)} target="_blank" rel="noopener noreferrer">
            <ExternalLink /> {t('nearby.searchOnMaps', { name: target.name })}
          </a>
        </Button>
        <p role="status" className="rounded-2xl bg-background px-4 py-3 text-sm text-ink-muted">
          {t('nearby.errorTitle')}: {localizedError(query.error, t, locale)}{' '}
          <button type="button" onClick={() => query.refetch()} className="font-semibold text-primary-700 hover:underline">
            {t('states.retry')}
          </button>
        </p>
      </div>
    );
  }

  const { places, is_mock, attribution, has_ratings } = query.data;
  const byRating = query.data.sort === 'rating';
  const first = places[0];

  return (
    <div className="space-y-3">
      <RadiusPicker />
      <div role="radiogroup" aria-label={t('nearby.sortAria')} className="grid grid-cols-2 gap-1 rounded-full bg-background p-1 ring-1 ring-border">
        {(['rating', 'distance'] as const).map((value) => (
          <button
            key={value}
            type="button"
            role="radio"
            aria-checked={sort === value}
            onClick={() => setSort(value)}
            className={cn(
              'flex items-center justify-center gap-1.5 rounded-full px-3 py-2 text-sm font-semibold transition',
              sort === value ? 'chip-active' : 'text-ink hover:bg-primary-50',
            )}
          >
            {value === 'rating' ? <Star className="size-4" /> : <MapPin className="size-4" />}
            {value === 'rating' ? t('nearby.sortRating') : t('nearby.sortDistance')}
          </button>
        ))}
      </div>
      {sort === 'rating' && !has_ratings && (
        <p className="flex items-start gap-2 rounded-2xl bg-background px-4 py-3 text-sm text-ink-muted">
          <Info className="mt-0.5 size-4 shrink-0" /> {t('nearby.needGoogle')}
        </p>
      )}
      {places.length > 0 && (
        <p className="text-sm text-ink-muted">{byRating ? t('nearby.ratingHint', { n: places.length }) : t('nearby.distanceHint', { n: places.length })}</p>
      )}

      {is_mock && (
        <p className="flex items-start gap-2 rounded-2xl bg-secondary/20 px-4 py-3 text-sm font-semibold text-ink">
          <TriangleAlert className="mt-0.5 size-[18px] shrink-0 text-secondary" />
          {t('nearby.mockNote')}
        </p>
      )}

      {first ? (
        <>
          <Button asChild size="lg" className="w-full">
            <a href={directionsUrl(first, location)} target="_blank" rel="noopener noreferrer">
              <Navigation />{' '}
              {byRating
                ? t('nearby.directionsTop', { d: formatDistance(first.distance_km) })
                : t('nearby.directionsNearest', { d: formatDistance(first.distance_km) })}
            </a>
          </Button>

          <ol className="space-y-2.5">
            {places.map((place, index) => {
              const isSaved = !!savedId(place);
              const top = byRating && index === 0 && place.rating != null;
              return (
                <li
                  key={`${place.provider}:${place.place_ref}`}
                  className={cn(
                    'flex items-center gap-3 rounded-2xl border bg-surface p-3.5',
                    top ? 'border-secondary/70 ring-2 ring-secondary/25' : 'border-border',
                  )}
                >
                  <div
                    className={cn(
                      'grid size-11 shrink-0 place-items-center rounded-full font-bold',
                      top ? 'bg-secondary/20 text-secondary' : 'bg-primary-50 text-primary',
                    )}
                    aria-hidden="true"
                  >
                    {top ? <Crown className="size-5" /> : byRating ? index + 1 : <MapPin className="size-5" />}
                  </div>
                  <div className="min-w-0 flex-1">
                    {top && <p className="text-xs font-bold uppercase tracking-wide text-secondary">{t('nearby.topPick')}</p>}
                    <p className="truncate font-semibold">{place.name}</p>
                    <p className="truncate text-sm text-ink-muted">{place.address ?? t('random.noAddress')}</p>
                    <p className="mt-0.5 flex flex-wrap items-center gap-x-3 text-sm font-medium">
                      <span className="text-primary-700">{formatDistance(place.distance_km)}</span>
                      {place.rating != null && (
                        <span className="flex items-center gap-1" title={place.rating_count != null ? t('nearby.reviews', { n: formatNumber(place.rating_count) }) : undefined}>
                          <Star className="size-3.5 fill-secondary text-secondary" />
                          {place.rating.toFixed(1)}
                          {place.rating_count != null && (
                            <span className="font-normal text-ink-muted">({formatCount(place.rating_count, formatNumber)})</span>
                          )}
                        </span>
                      )}
                      {place.open_now != null && (
                        <span className={place.open_now ? 'text-leaf' : 'text-ink-soft'}>{place.open_now ? t('nearby.openNow') : t('nearby.closedNow')}</span>
                      )}
                    </p>
                  </div>
                  <button
                    type="button"
                    onClick={() => toggle(place, target)}
                    aria-pressed={isSaved}
                    aria-label={isSaved ? t('nearby.unsave', { name: place.name }) : t('nearby.save', { name: place.name })}
                    className="grid size-10 shrink-0 place-items-center rounded-full text-ink-muted transition hover:bg-primary-50 hover:text-primary"
                  >
                    <Heart className={cn('size-5', isSaved && 'fill-primary text-primary')} />
                  </button>
                  <Button asChild variant="soft" size="sm" className="shrink-0">
                    <a href={directionsUrl(place, location)} target="_blank" rel="noopener noreferrer" aria-label={t('nearby.directionsTo', { name: place.name })}>
                      <Navigation /> <span className="hidden sm:inline">{t('nearby.directions')}</span>
                    </a>
                  </Button>
                </li>
              );
            })}
          </ol>
        </>
      ) : (
        <div className="rounded-2xl border border-dashed border-border px-4 py-6 text-center text-[15px] text-ink-muted">
          <p>{t('nearby.noPlaces', { cuisine: t(`options.cuisines.${target.cuisine}`) })}</p>
          {widerRadius != null && (
            <Button variant="soft" size="sm" className="mt-3" onClick={() => setRadius(widerRadius)}>
              {t('nearby.widen', { km: widerRadius })}
            </Button>
          )}
        </div>
      )}

      {mapsSearch}
      <p className="text-center text-xs text-ink-soft">
        {attribution} · {t('nearby.straightLine')}
      </p>
    </div>
  );
}
