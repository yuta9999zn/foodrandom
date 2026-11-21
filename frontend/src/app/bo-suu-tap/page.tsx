'use client';

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Heart, MapPin, Navigation, Trash2 } from 'lucide-react';
import Link from 'next/link';
import { useState } from 'react';
import { toast } from 'sonner';

import { FoodCard, FoodCardSkeleton, FoodGrid } from '@/components/food/food-card';
import { EmptyState, ErrorState } from '@/components/states';
import { Button } from '@/components/ui/button';
import { useMe } from '@/hooks/use-auth';
import { usePrefs } from '@/hooks/use-favorites';
import { useI18n } from '@/i18n/provider';
import { favoritesApi, localizedError } from '@/lib/api';
import { directionsUrl } from '@/lib/maps';
import { cn } from '@/lib/utils';
import { useLocationStore } from '@/stores/location-store';

function SavedFoods() {
  const { t, locale } = useI18n();
  const prefs = usePrefs();
  const query = useQuery({ queryKey: ['favorites', 'foods', prefs], queryFn: async () => (await favoritesApi.foods(prefs)).items });
  if (query.isPending)
    return (
      <FoodGrid>
        {Array.from({ length: 4 }, (_, i) => (
          <FoodCardSkeleton key={i} />
        ))}
      </FoodGrid>
    );
  if (query.isError) return <ErrorState title={t('states.loadError')} message={localizedError(query.error, t, locale)} onRetry={() => query.refetch()} retryLabel={t('states.retry')} />;
  if (query.data.length === 0) return <EmptyState title={t('collection.foods')} description={t('collection.emptyFoods')} icon={<Heart className="size-8" />} />;
  return (
    <FoodGrid>
      {query.data.map((food) => (
        <FoodCard key={food.id} food={food} />
      ))}
    </FoodGrid>
  );
}

function SavedPlaces() {
  const { t, locale } = useI18n();
  const queryClient = useQueryClient();
  const origin = useLocationStore((s) => s.location);
  const query = useQuery({ queryKey: ['favorites', 'places'], queryFn: async () => (await favoritesApi.places()).items });
  const remove = useMutation({
    mutationFn: favoritesApi.removePlace,
    onSuccess: () => {
      toast.success(t('nearby.unsaved'));
      queryClient.invalidateQueries({ queryKey: ['favorites', 'places'] });
    },
    onError: (error) => toast.error(localizedError(error, t, locale)),
  });

  if (query.isPending) return <div className="space-y-3">{[0, 1, 2].map((i) => <div key={i} className="skeleton h-24" />)}</div>;
  if (query.isError) return <ErrorState title={t('states.loadError')} message={localizedError(query.error, t, locale)} onRetry={() => query.refetch()} retryLabel={t('states.retry')} />;
  if (query.data.length === 0) return <EmptyState title={t('collection.places')} description={t('collection.emptyPlaces')} icon={<MapPin className="size-8" />} />;

  return (
    <ul className="grid gap-3 md:grid-cols-2">
      {query.data.map((place) => (
        <li key={place.id} className="flex items-start gap-3 rounded-2xl bg-surface p-4 shadow-card ring-1 ring-border">
          <div className="grid size-11 shrink-0 place-items-center rounded-full bg-primary-50 text-primary">
            <MapPin className="size-5" />
          </div>
          <div className="min-w-0 flex-1">
            <p className="font-semibold">{place.name}</p>
            <p className="text-sm text-ink-muted">{place.address ?? t('random.noAddress')}</p>
            {place.food_slug && place.food_name && (
              <Link href={`/foods/${place.food_slug}`} className="mt-1 inline-block text-sm font-medium text-primary-700 hover:underline">
                {t('collection.fromFood', { name: place.food_name })}
              </Link>
            )}
            <div className="mt-3 flex gap-2">
              <Button asChild size="sm" variant="soft">
                <a href={directionsUrl(place, origin)} target="_blank" rel="noopener noreferrer">
                  <Navigation /> {t('collection.directions')}
                </a>
              </Button>
              <Button size="sm" variant="ghost" onClick={() => remove.mutate(place.id)} disabled={remove.isPending}>
                <Trash2 /> {t('collection.remove')}
              </Button>
            </div>
          </div>
        </li>
      ))}
    </ul>
  );
}

export default function CollectionPage() {
  const { t } = useI18n();
  const { data: me, isPending } = useMe();
  const [tab, setTab] = useState<'foods' | 'places'>('foods');

  return (
    <div className="container py-10">
      <h1 className="text-3xl font-bold tracking-tight">{t('collection.title')}</h1>
      <p className="mt-1 text-ink-muted">{t('collection.subtitle')}</p>

      {isPending ? (
        <div className="skeleton mt-8 h-64" />
      ) : !me ? (
        <EmptyState
          className="mt-8"
          title={t('collection.title')}
          description={t('collection.needLogin')}
          icon={<Heart className="size-8" />}
          action={
            <Button asChild>
              <Link href="/dang-nhap?next=/bo-suu-tap">{t('nav.login')}</Link>
            </Button>
          }
        />
      ) : (
        <>
          <div role="tablist" className="mt-6 inline-flex rounded-full bg-surface p-1 ring-1 ring-border">
            {(['foods', 'places'] as const).map((key) => (
              <button
                key={key}
                role="tab"
                aria-selected={tab === key}
                onClick={() => setTab(key)}
                className={cn('rounded-full px-5 py-2 text-[15px] font-semibold transition', tab === key ? 'chip-active' : 'text-ink hover:bg-primary-50')}
              >
                {t(`collection.${key}`)}
              </button>
            ))}
          </div>
          <div className="mt-6" role="tabpanel">
            {tab === 'foods' ? <SavedFoods /> : <SavedPlaces />}
          </div>
        </>
      )}
    </div>
  );
}
